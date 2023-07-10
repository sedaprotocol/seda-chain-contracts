#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Order, Response, StdResult};

use crate::state::{DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL};

use crate::msg::{GetDataRequestResponse, GetDataRequestsResponse};
use crate::state::DataRequest;

use crate::ContractError;

pub mod data_requests {
    use cw_storage_plus::Bound;

    use super::*;

    /// Posts a data request to the pool
    pub fn post_data_request(
        deps: DepsMut,
        _info: MessageInfo,
        value: String,
    ) -> Result<Response, ContractError> {
        // save the data request
        let dr_id = DATA_REQUESTS_COUNT.load(deps.storage)?;
        DATA_REQUESTS_POOL.save(deps.storage, dr_id, &DataRequest { value, dr_id })?;

        // increment the data request count
        DATA_REQUESTS_COUNT.update(deps.storage, |mut new_dr_id| -> Result<_, ContractError> {
            new_dr_id += 1;
            Ok(new_dr_id)
        })?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .add_attribute("dr_id", dr_id.to_string()))
    }

    /// Returns a data request from the pool with the given id, if it exists.
    pub fn get_data_request(deps: Deps, dr_id: u128) -> StdResult<GetDataRequestResponse> {
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
        for dr in DATA_REQUESTS_POOL.range(
            deps.storage,
            Some(Bound::InclusiveRaw(position.into())),
            Some(Bound::ExclusiveRaw(dr_count.into())),
            Order::Ascending,
        ) {
            requests.push(dr?.1);
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

        // data request with id 0 does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data request with id 0
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(DataRequest {
                dr_id: 0 as u128,
                value: "hello world".to_string()
            }),
            value.value
        );

        // data request with id 1 does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest { dr_id: 1 },
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
            value: "0".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "1".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let msg = ExecuteMsg::PostDataRequest {
            value: "2".to_string(),
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
                        dr_id: 0 as u128,
                        value: "0".to_string()
                    },
                    DataRequest {
                        dr_id: 1 as u128,
                        value: "1".to_string()
                    },
                    DataRequest {
                        dr_id: 2 as u128,
                        value: "2".to_string()
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
                        dr_id: 0 as u128,
                        value: "0".to_string()
                    },
                    DataRequest {
                        dr_id: 1 as u128,
                        value: "1".to_string()
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
                    dr_id: 1 as u128,
                    value: "1".to_string()
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
                        dr_id: 1 as u128,
                        value: "1".to_string()
                    },
                    DataRequest {
                        dr_id: 2 as u128,
                        value: "2".to_string()
                    },
                ]
            },
            response
        );
    }
}
