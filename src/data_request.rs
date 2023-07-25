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
    pub fn post_data_request(
        deps: DepsMut,
        _info: MessageInfo,
        dr_id: Hash,
        value: String,
        nonce: u128,
        chain_id: u128,
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
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request with id 0x66... does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data request with id 0x66...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae"
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
                dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae"
                    .to_string(),
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

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone posts three data requests
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "1".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x1ae35ab1000b2d88156730481e1e913e8b78a980e956a5eb4221f1e3403887dd".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "2".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x71847d61bd26ef76413a7b766bbdbcfde4724b09d19bbd01432c478cbcf71162".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "3".to_string(),
            nonce: 1,
            chain_id: 31337,
            dr_id: "0x1a1b26ca0700551abdaf41ed3199dbd14ac9bb70cfb4e58d83e1f3bf2c6d548a".to_string(),
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
                        dr_id: "0x1ae35ab1000b2d88156730481e1e913e8b78a980e956a5eb4221f1e3403887dd"
                            .to_string()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x71847d61bd26ef76413a7b766bbdbcfde4724b09d19bbd01432c478cbcf71162"
                            .to_string()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x1a1b26ca0700551abdaf41ed3199dbd14ac9bb70cfb4e58d83e1f3bf2c6d548a"
                            .to_string()
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
                        dr_id: "0x1ae35ab1000b2d88156730481e1e913e8b78a980e956a5eb4221f1e3403887dd"
                            .to_string()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x71847d61bd26ef76413a7b766bbdbcfde4724b09d19bbd01432c478cbcf71162"
                            .to_string()
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
                    dr_id: "0x71847d61bd26ef76413a7b766bbdbcfde4724b09d19bbd01432c478cbcf71162"
                        .to_string()
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
                        dr_id: "0x71847d61bd26ef76413a7b766bbdbcfde4724b09d19bbd01432c478cbcf71162"
                            .to_string()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1,
                        chain_id: 31337,
                        dr_id: "0x1a1b26ca0700551abdaf41ed3199dbd14ac9bb70cfb4e58d83e1f3bf2c6d548a"
                            .to_string()
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
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
            result: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // can't create a data request with the same id as a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());
    }
}
