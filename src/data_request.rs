#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Order, Response, StdResult};
use ethers::utils::keccak256;

use crate::state::{DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL};

use crate::msg::{GetDataRequestResponse, GetDataRequestsResponse};
use crate::state::DataRequest;
use crate::types::Hash;

use crate::ContractError;

pub mod data_requests {
    use cw_storage_plus::Bound;

    use crate::state::DATA_REQUESTS_BY_NONCE;

    use super::*;

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
        if DATA_REQUESTS_POOL
            .may_load(deps.storage, dr_id.clone())?
            .is_some()
        {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // reconstruct the data request id hash
        // TODO: make generic, remove ethers dependency
        let encoded_params = ethers::abi::encode(&[
            ethers::abi::Token::Uint(nonce.into()),
            ethers::abi::Token::String(value.clone()),
            ethers::abi::Token::Uint(chain_id.into()),
        ]);
        let reconstructed_dr_id = format!("0x{}", hex::encode(keccak256(encoded_params)));

        // check if the reconstructed dr_id matches the given dr_id
        if reconstructed_dr_id != dr_id {
            return Err(ContractError::InvalidDataRequestId(
                dr_id,
                reconstructed_dr_id,
            ));
        }

        // check if the dr_id is already in the pool
        if DATA_REQUESTS_POOL
            .may_load(deps.storage, dr_id.clone())?
            .is_some()
        {
            return Err(ContractError::DataRequestAlreadyExists);
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
    pub fn get_data_requests(
        deps: Deps,
        position: Option<u128>,
        limit: Option<u32>,
    ) -> StdResult<GetDataRequestsResponse> {
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
            requests.push(DATA_REQUESTS_POOL.load(deps.storage, dr?.1)?);
            if requests.len() == limit as usize {
                break;
            }
        }

        Ok(GetDataRequestsResponse { value: requests })
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
                dr_id: "0x6602112640959ba080ae4cc0861e56fc70d5261cffddc1f016091aebc60f4063"
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
            chain_id: 31337 as u128,
            nonce: 1 as u128,
            dr_id: "0x6602112640959ba080ae4cc0861e56fc70d5261cffddc1f016091aebc60f4063".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data request with id 0x66...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x6602112640959ba080ae4cc0861e56fc70d5261cffddc1f016091aebc60f4063"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(DataRequest {
                value: "hello world".to_string(),
                chain_id: 31337 as u128,
                nonce: 1 as u128,
                dr_id: "0x6602112640959ba080ae4cc0861e56fc70d5261cffddc1f016091aebc60f4063"
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
            nonce: 1 as u128,
            chain_id: 31337 as u128,
            dr_id: "0x3855afc167b4429c3b05600cc16ef5f30d5ee7fb5c56805ab295488abd270014".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "2".to_string(),
            nonce: 1 as u128,
            chain_id: 31337 as u128,
            dr_id: "0x20aabf330be2a6a2510c4880c3f0e28e7e8cec33f38e05094f6ec7070ea4297a".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "3".to_string(),
            nonce: 1 as u128,
            chain_id: 31337 as u128,
            dr_id: "0x95b51ee8670e9ab36daa83281c7531e13d0a2f6b0992c5e55d8622d379562d87".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetch all three data requests
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequests {
                position: None,
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "1".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x3855afc167b4429c3b05600cc16ef5f30d5ee7fb5c56805ab295488abd270014"
                            .to_string()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x20aabf330be2a6a2510c4880c3f0e28e7e8cec33f38e05094f6ec7070ea4297a"
                            .to_string()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x95b51ee8670e9ab36daa83281c7531e13d0a2f6b0992c5e55d8622d379562d87"
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
            QueryMsg::GetDataRequests {
                position: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "1".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x3855afc167b4429c3b05600cc16ef5f30d5ee7fb5c56805ab295488abd270014"
                            .to_string()
                    },
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x20aabf330be2a6a2510c4880c3f0e28e7e8cec33f38e05094f6ec7070ea4297a"
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
            QueryMsg::GetDataRequests {
                position: Some(1),
                limit: Some(1),
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![DataRequest {
                    value: "2".to_string(),
                    nonce: 1 as u128,
                    chain_id: 31337 as u128,
                    dr_id: "0x20aabf330be2a6a2510c4880c3f0e28e7e8cec33f38e05094f6ec7070ea4297a"
                        .to_string()
                },]
            },
            response
        );

        // fetch all data requests starting from id 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequests {
                position: Some(1),
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "2".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x20aabf330be2a6a2510c4880c3f0e28e7e8cec33f38e05094f6ec7070ea4297a"
                            .to_string()
                    },
                    DataRequest {
                        value: "3".to_string(),
                        nonce: 1 as u128,
                        chain_id: 31337 as u128,
                        dr_id: "0x95b51ee8670e9ab36daa83281c7531e13d0a2f6b0992c5e55d8622d379562d87"
                            .to_string()
                    },
                ]
            },
            response
        );
    }
}
