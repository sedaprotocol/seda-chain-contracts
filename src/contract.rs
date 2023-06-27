#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetDataRequestExecutorResponse, GetDataRequestResponse, GetDataRequestsResponse,
    GetDataResultResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{
    DataRequest, DataRequestExecutor, DataResult, DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL,
    DATA_RESULTS, INACTIVE_DATA_REQUEST_EXECUTORS, TOKEN,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DATA_REQUESTS_COUNT.save(deps.storage, &0)?;
    TOKEN.save(deps.storage, &msg.token)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostDataRequest { value } => execute::post_data_request(deps, info, value),
        ExecuteMsg::PostDataResult { dr_id, result } => {
            execute::post_data_result(deps, info, dr_id, result)
        }
        ExecuteMsg::RegisterDataRequestExecutor {
            bn254_public_key,
            multi_address,
        } => execute::register_data_request_executor(deps, info, bn254_public_key, multi_address),
        ExecuteMsg::UnregisterDataRequestExecutor {} => {
            execute::unregister_data_request_executor(deps, info)
        }
        ExecuteMsg::Stake => execute::stake(deps, env, info),
        ExecuteMsg::Unstake { amount } => execute::unstake(deps, env, info, amount),
    }
}

pub mod execute {

    use cosmwasm_std::{coins, BankMsg};

    use crate::state::DATA_RESULTS;

    use super::*;

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

    pub fn post_data_result(
        deps: DepsMut,
        _info: MessageInfo,
        dr_id: u128,
        result: String,
    ) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let dr_result = DataResult {
            dr_id: dr.dr_id,
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

    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        bn254_public_key: String,
        multi_address: String,
    ) -> Result<Response, ContractError> {
        // TODO: require token deposit
        // TODO: verify bn254_public_key using signature

        let executor = DataRequestExecutor {
            bn254_public_key: bn254_public_key.clone(),
            multi_address: multi_address.clone(),
            staked_tokens: 0,
        };
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;

        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_attribute("executor", info.sender)
            .add_attribute("bn254_public_key", bn254_public_key)
            .add_attribute("multi_address", multi_address))
    }

    pub fn unregister_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        INACTIVE_DATA_REQUEST_EXECUTORS.remove(deps.storage, info.sender.clone());

        Ok(Response::new()
            .add_attribute("action", "unregister_data_request_executor")
            .add_attribute("executor", info.sender))
    }

    pub fn stake(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;

        // get amount of tokens from sender
        let amount: Option<u128> = info
            .funds
            .iter()
            .find(|coin| coin.denom == token)
            .map(|coin| coin.amount.u128());

        // assert that the sender has enough tokens
        // TODO: set to non-zero minimum
        let amount = amount.ok_or(ContractError::InsufficientFunds)?;

        // update staked tokens for executor
        let mut executor =
            INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.clone().sender)?;
        executor.staked_tokens += amount;
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.clone().sender, &executor)?;

        Ok(Response::new()
            .add_attribute("action", "stake")
            .add_attribute("executor", info.sender)
            .add_attribute("amount", amount.to_string()))
    }

    pub fn unstake(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: u128,
    ) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than staked tokens
        let mut executor =
            INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.sender.clone())?;
        if amount > executor.staked_tokens {
            return Err(ContractError::InsufficientFunds);
        }

        // update the executor
        executor.staked_tokens -= amount;
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: coins(amount, token),
        };

        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "unstake")
            .add_attribute("executor", info.sender)
            .add_attribute("amount", amount.to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequest { dr_id } => to_binary(&query::get_data_request(deps, dr_id)?),
        QueryMsg::GetDataRequests { position, limit } => {
            to_binary(&query::get_data_requests(deps, position, limit)?)
        }
        QueryMsg::GetDataResult { dr_id } => to_binary(&query::get_data_result(deps, dr_id)?),
        QueryMsg::GetDataRequestExecutor { executor } => {
            to_binary(&query::get_data_request_executor(deps, executor)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::Addr;
    use cw_storage_plus::Bound;

    use super::*;

    pub fn get_data_request(deps: Deps, dr_id: u128) -> StdResult<GetDataRequestResponse> {
        let dr = DATA_REQUESTS_POOL.may_load(deps.storage, dr_id)?;
        Ok(GetDataRequestResponse { value: dr })
    }

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

    pub fn get_data_result(deps: Deps, dr_id: u128) -> StdResult<GetDataResultResponse> {
        let dr = DATA_RESULTS.may_load(deps.storage, dr_id)?;
        Ok(GetDataResultResponse { value: dr })
    }

    pub fn get_data_request_executor(
        deps: Deps,
        executor: Addr,
    ) -> StdResult<GetDataRequestExecutorResponse> {
        let executor = INACTIVE_DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(GetDataRequestExecutorResponse { value: executor })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

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
    fn post_data_result() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
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
                dr_id: 0 as u128,
                value: "hello world".to_string(),
                result: "dr 0 result".to_string()
            }),
            value.value
        );
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

    #[test]
    fn register_data_request_executor() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetching data request executor for an address that doesn't exist should return None
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(value, GetDataRequestExecutorResponse { value: None });

        // someone registers a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            bn254_public_key: "key".to_string(),
            multi_address: "address".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch the data request executor
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    bn254_public_key: "key".to_string(),
                    multi_address: "address".to_string(),
                    staked_tokens: 0
                })
            }
        );
    }

    #[test]
    fn unregister_data_request_executor() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone registers a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            bn254_public_key: "key".to_string(),
            multi_address: "address".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch the data request executor
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    bn254_public_key: "key".to_string(),
                    multi_address: "address".to_string(),
                    staked_tokens: 0
                })
            }
        );

        // unregister the data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UnregisterDataRequestExecutor {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetching data request executor after unregistering should return None
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(value, GetDataRequestExecutorResponse { value: None });
    }

    #[test]
    fn stake() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(0, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // register a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            bn254_public_key: "key".to_string(),
            multi_address: "address".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // the data request executor stakes 2 tokens
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Stake;
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 2
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    bn254_public_key: "key".to_string(),
                    multi_address: "address".to_string(),
                    staked_tokens: 2
                })
            }
        );

        // the data request executor unstakes 1
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake { amount: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    bn254_public_key: "key".to_string(),
                    multi_address: "address".to_string(),
                    staked_tokens: 1
                })
            }
        );
    }
}
