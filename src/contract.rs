#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetDataRequestResponse, GetDataRequestsResponse, GetDataResultResponse,
    InstantiateMsg, QueryMsg,
};
use crate::state::{
    DataRequest, DataResult, DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL, DATA_RESULTS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DATA_REQUESTS_COUNT.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostDataRequest { value } => execute::post_data_request(deps, info, value),
        ExecuteMsg::PostDataResult { dr_id, result } => {
            execute::post_data_result(deps, info, dr_id, result)
        }
    }
}

pub mod execute {

    use crate::state::DATA_RESULTS;

    use super::*;

    pub fn post_data_request(
        deps: DepsMut,
        _info: MessageInfo,
        value: String,
    ) -> Result<Response, ContractError> {
        // save the data request
        let dr_id = DATA_REQUESTS_COUNT.load(deps.storage)?;
        DATA_REQUESTS_POOL.save(deps.storage, dr_id, &DataRequest { value })?;

        // increment the data request count
        DATA_REQUESTS_COUNT.update(deps.storage, |mut new_dr_id| -> Result<_, ContractError> {
            new_dr_id += 1;
            Ok(new_dr_id)
        })?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .add_attribute("dr_id", dr_id.to_string()))
    }

    pub fn post_data_result(
        deps: DepsMut,
        _info: MessageInfo,
        dr_id: u128,
        result: String,
    ) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let dr_result = DataResult {
            value: dr.value,
            result: result.clone(),
        };

        // save the data result then remove it from the pool
        DATA_RESULTS.save(deps.storage, dr_id, &dr_result)?;
        DATA_REQUESTS_POOL.remove(deps.storage, dr_id);

        Ok(Response::new()
            .add_attribute("action", "post_data_result")
            .add_attribute("dr_id", dr_id.to_string())
            .add_attribute("result", result))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequest { dr_id } => to_binary(&query::get_data_request(deps, dr_id)?),
        QueryMsg::GetDataRequests { start_dr_id, limit } => {
            to_binary(&query::get_data_requests(deps, start_dr_id, limit)?)
        }
        QueryMsg::GetDataResult { dr_id } => to_binary(&query::get_data_result(deps, dr_id)?),
    }
}

pub mod query {
    use cw_storage_plus::Bound;

    use super::*;

    pub fn get_data_request(deps: Deps, dr_id: u128) -> StdResult<GetDataRequestResponse> {
        let dr = DATA_REQUESTS_POOL.may_load(deps.storage, dr_id)?;
        Ok(GetDataRequestResponse { value: dr })
    }

    pub fn get_data_requests(
        deps: Deps,
        start_dr_id: Option<u128>,
        limit: Option<u32>,
    ) -> StdResult<GetDataRequestsResponse> {
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?.to_be_bytes();
        let start_dr_id = start_dr_id.unwrap_or(0).to_be_bytes();
        let limit = limit.unwrap_or(u32::MAX);

        // starting from start_dr_id, iterate forwards until we reach the limit or the end of the data requests
        let mut requests = vec![];
        for dr in DATA_REQUESTS_POOL.range(
            deps.storage,
            Some(Bound::InclusiveRaw(start_dr_id.into())),
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

    pub fn get_data_result(deps: Deps, dr_id: u128) -> StdResult<GetDataResultResponse> {
        let dr = DATA_RESULTS.may_load(deps.storage, dr_id)?;
        Ok(GetDataResultResponse { value: dr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn post_data_request() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
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
    fn post_data_result() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // can't post a data result for a data request that doesn't exist
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: 0 as u128,
            result: "dr 0 result".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data result with id 0 does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataResult { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: 0 as u128,
            result: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data result with id 0
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataResult { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataResultResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(DataResult {
                value: "hello world".to_string(),
                result: "dr 0 result".to_string()
            }),
            value.value
        );
    }

    #[test]
    fn get_data_requests() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
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
                start_dr_id: None,
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "0".to_string()
                    },
                    DataRequest {
                        value: "1".to_string()
                    },
                    DataRequest {
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
                start_dr_id: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "0".to_string()
                    },
                    DataRequest {
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
                start_dr_id: Some(1),
                limit: Some(1),
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![DataRequest {
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
                start_dr_id: Some(1),
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsResponse {
                value: vec![
                    DataRequest {
                        value: "1".to_string()
                    },
                    DataRequest {
                        value: "2".to_string()
                    },
                ]
            },
            response
        );
    }
}
