#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Order, Response, StdResult};
use sha3::{Digest, Keccak256};

use crate::state::{DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL, DATA_RESULTS};

use crate::helpers::hash_update;
use crate::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};
use crate::state::DataRequest;
use crate::types::Hash;

use crate::ContractError;

pub mod data_requests {
    use cw_storage_plus::Bound;

    use crate::state::DATA_REQUESTS_BY_NONCE;

    use super::*;

    /// Internal function to return whether a data request or result exists with the given id.
    fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
        DATA_REQUESTS_POOL
            .may_load(deps.storage, dr_id.clone())
            .ok()
            .flatten()
            .is_some()
            || DATA_RESULTS
                .may_load(deps.storage, dr_id)
                .ok()
                .flatten()
                .is_some()
    }

    /// Posts a data request to the pool
    #[allow(clippy::too_many_arguments)]
    pub fn post_data_request(
        deps: DepsMut,
        _info: MessageInfo,
        dr_id: Hash,
        value: String,
        nonce: u128,
        chain_id: u128,
        wasm_id: Vec<u8>,
        wasm_args: Vec<Vec<u8>>,
    ) -> Result<Response, ContractError> {
        // require the data request id to be unique
        if data_request_or_result_exists(deps.as_ref(), dr_id.clone()) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // reconstruct the data request id hash
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, nonce);
        hasher.update(value.as_bytes());
        hash_update(&mut hasher, chain_id);
        hasher.update(wasm_id.as_slice());
        for arg in wasm_args.iter() {
            hasher.update(arg.as_slice());
        }
        let reconstructed_dr_id_bytes = hasher.finalize();
        let reconstructed_dr_id = format!("0x{}", hex::encode(reconstructed_dr_id_bytes));

        // check if the reconstructed dr_id matches the given dr_id
        if reconstructed_dr_id != dr_id {
            return Err(ContractError::InvalidDataRequestId(
                reconstructed_dr_id,
                dr_id,
            ));
        }

        // save the data request
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?;
        DATA_REQUESTS_POOL.save(
            deps.storage,
            dr_id.clone(),
            &DataRequest {
                value,
                dr_id: dr_id.clone(),
                nonce,
                chain_id,
                wasm_id,
                wasm_args,
            },
        )?;
        DATA_REQUESTS_BY_NONCE.save(deps.storage, dr_count, &dr_id)?; // todo wrong nonce

        // increment the data request count
        DATA_REQUESTS_COUNT.update(deps.storage, |mut new_dr_id| -> Result<_, ContractError> {
            new_dr_id += 1;
            Ok(new_dr_id)
        })?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .add_attribute("dr_id", dr_id))
    }

    /// Returns a data request from the pool with the given id, if it exists.
    pub fn get_data_request(deps: Deps, dr_id: Hash) -> StdResult<GetDataRequestResponse> {
        let dr = DATA_REQUESTS_POOL.may_load(deps.storage, dr_id)?;
        Ok(GetDataRequestResponse { value: dr })
    }

    /// Returns a list of data requests from the pool, starting from the given position and limited by the given limit.
    pub fn get_data_requests_from_pool(
        deps: Deps,
        position: Option<u128>,
        limit: Option<u32>,
    ) -> StdResult<GetDataRequestsFromPoolResponse> {
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?.to_be_bytes();
        let position = position.unwrap_or(0).to_be_bytes();
        let limit = limit.unwrap_or(u32::MAX);

        // starting from position, iterate forwards until we reach the limit or the end of the data requests
        let mut requests = vec![];
        for dr in DATA_REQUESTS_BY_NONCE.range(
            deps.storage,
            Some(Bound::InclusiveRaw(position.into())),
            Some(Bound::ExclusiveRaw(dr_count.into())),
            Order::Ascending,
        ) {
            let dr_pending = DATA_REQUESTS_POOL.may_load(deps.storage, dr?.1)?;
            // skip if the data request is no longer in the pool
            if dr_pending.is_none() {
                continue;
            }
            requests.push(dr_pending.unwrap());
            if requests.len() == limit as usize {
                break;
            }
        }

        Ok(GetDataRequestsFromPoolResponse { value: requests })
    }
}

#[cfg(test)]
mod dr_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::query;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    use crate::contract::instantiate;
    use crate::msg::GetDataRequestResponse;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn post_data_request() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            wasm_storage_contract_address: "wasm_storage_contract_address".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request with id 0x69... does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // set arguments for post_data_request
        // TODO: move this and duplicates to a helper function
        let wasm_id = "wasm_id".to_string().into_bytes();
        let mut wasm_args: Vec<Vec<u8>> = vec![];
        wasm_args.push("arg1".to_string().into_bytes());
        wasm_args.push("arg2".to_string().into_bytes());

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data request with id 0x69...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(DataRequest {
                value: "hello world".to_string(),
                chain_id: 31337,
                nonce: 1,
                dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f"
                    .to_string(),
                wasm_id,
                wasm_args
            }),
            value.value
        );

        // nonexistent data request does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "nonexistent".to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);
    }

    #[test]
    fn get_data_requests() {
        let mut deps = mock_dependencies();

        // instantiate contract
        let msg = InstantiateMsg {
            token: "token".to_string(),
            wasm_storage_contract_address: "wasm_storage_contract_address".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // set arguments for data request
        let wasm_id = "wasm_id".to_string().into_bytes();
        let mut wasm_args: Vec<Vec<u8>> = vec![];
        wasm_args.push("arg1".to_string().into_bytes());
        wasm_args.push("arg2".to_string().into_bytes());

        // someone posts three data requests
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "1".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x11016abf1a828a71787dfb403b76c3ddb9aa1d80f9b9ea5748e48d2c10a38777".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "2".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x163109688c51e6c41c9db047e5fb1f6be92bf250f39d3efc62448b45a1211019".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "3".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x03fa1e34b70fadffd926f685d8195bc590636702a03d217b322d5c229683fdf0".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetch all three data requests
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        value: "1".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x11016abf1a828a71787dfb403b76c3ddb9aa1d80f9b9ea5748e48d2c10a38777"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x163109688c51e6c41c9db047e5fb1f6be92bf250f39d3efc62448b45a1211019"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x03fa1e34b70fadffd926f685d8195bc590636702a03d217b322d5c229683fdf0"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                ]
            },
            response
        );

        // fetch data requests with limit of 2
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        value: "1".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x11016abf1a828a71787dfb403b76c3ddb9aa1d80f9b9ea5748e48d2c10a38777"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x163109688c51e6c41c9db047e5fb1f6be92bf250f39d3efc62448b45a1211019"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone(),
                    },
                ]
            },
            response
        );

        // fetch a single data request
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: Some(1),
                limit: Some(1),
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![DataRequest {
                    value: "2".to_string(),
                    nonce: 1,
                    chain_id: 31337,
                    dr_id: "0x163109688c51e6c41c9db047e5fb1f6be92bf250f39d3efc62448b45a1211019"
                        .to_string(),
                    wasm_id: wasm_id.clone(),
                    wasm_args: wasm_args.clone(),
                },]
            },
            response
        );

        // fetch all data requests starting from id 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: Some(1),
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x163109688c51e6c41c9db047e5fb1f6be92bf250f39d3efc62448b45a1211019"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x03fa1e34b70fadffd926f685d8195bc590636702a03d217b322d5c229683fdf0"
                            .to_string(),
                        wasm_id: wasm_id.clone(),
                        wasm_args: wasm_args.clone()
                    },
                ]
            },
            response
        );
    }

    #[test]
    fn no_duplicate_dr_ids() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            wasm_storage_contract_address: "wasm_storage_contract_address".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // register dr executor
        let info = mock_info("anyone", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // set arguments for post_data_request
        // TODO: move this and duplicates to a helper function
        let wasm_id = "wasm_id".to_string().into_bytes();
        let mut wasm_args: Vec<Vec<u8>> = vec![];
        wasm_args.push("arg1".to_string().into_bytes());
        wasm_args.push("arg2".to_string().into_bytes());

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            result: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // can't create a data request with the same id as a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            wasm_id: wasm_id,
            wasm_args: wasm_args,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());
    }
}
