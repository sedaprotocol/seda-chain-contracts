#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::TOKEN;
use crate::utils::get_attached_funds;
use common::{
    error::ContractError,
    msg::{
        DataRequestsExecuteMsg, GetCommittedDataResultResponse, GetCommittedDataResultsResponse,
        GetContractResponse, GetDataRequestExecutorResponse, GetDataRequestResponse,
        GetDataRequestsFromPoolResponse, GetResolvedDataResultResponse,
        GetRevealedDataResultResponse, GetRevealedDataResultsResponse,
        IsDataRequestExecutorEligibleResponse, StakingExecuteMsg,
    },
};
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::{
    msg::{InstantiateMsg, ProxyExecuteMsg, ProxyQueryMsg, ProxySudoMsg},
    state::{DATA_REQUESTS, STAKING},
};

// version info
const CONTRACT_NAME: &str = "proxy-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TOKEN.save(deps.storage, &msg.token)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ProxyExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Admin
        ProxyExecuteMsg::SetDataRequests { contract } => {
            // This can only be called if not already set. Otherwise, a sudo message must be used.
            // if already set, return error
            if DATA_REQUESTS.may_load(deps.storage)?.is_some() {
                return Err(ContractError::ContractAlreadySet {});
            }

            DATA_REQUESTS.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            Ok(Response::new().add_attribute("method", "set_data_requests"))
        }
        ProxyExecuteMsg::SetStaking { contract } => {
            // This can only be called if not already set. Otherwise, a sudo message must be used.
            // if already set, return error
            if STAKING.may_load(deps.storage)?.is_some() {
                return Err(ContractError::ContractAlreadySet {});
            }

            STAKING.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            Ok(Response::new().add_attribute("method", "set_staking"))
        }

        // Delegated calls to contracts

        // DataRequests
        ProxyExecuteMsg::PostDataRequest { posted_dr } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                msg: to_binary(&DataRequestsExecuteMsg::PostDataRequest { posted_dr })?,
                funds: vec![],
            }))
            .add_attribute("action", "post_data_request")),
        ProxyExecuteMsg::CommitDataResult { dr_id, commitment } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                msg: to_binary(&DataRequestsExecuteMsg::CommitDataResult {
                    dr_id,
                    commitment,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "commit_data_result")),
        ProxyExecuteMsg::RevealDataResult { dr_id, reveal } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                msg: to_binary(&DataRequestsExecuteMsg::RevealDataResult {
                    dr_id,
                    reveal,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "reveal_data_result")),

        // Staking
        ProxyExecuteMsg::RegisterDataRequestExecutor { p2p_multi_address } => {
            // require token deposit
            let token = TOKEN.load(deps.storage)?;
            let amount = get_attached_funds(&info.funds, &token)?;

            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: STAKING.load(deps.storage)?.to_string(),
                    msg: to_binary(&StakingExecuteMsg::RegisterDataRequestExecutor {
                        p2p_multi_address,
                        sender: Some(info.sender.to_string()),
                    })?,
                    funds: vec![Coin {
                        denom: token,
                        amount: amount.into(),
                    }],
                }))
                .add_attribute("action", "register_data_request_executor"))
        }
        ProxyExecuteMsg::UnregisterDataRequestExecutor {} => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: STAKING.load(deps.storage)?.to_string(),
                msg: to_binary(&StakingExecuteMsg::UnregisterDataRequestExecutor {
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "unregister_data_request_executor")),
        ProxyExecuteMsg::DepositAndStake {} => {
            // require token deposit
            let token = TOKEN.load(deps.storage)?;
            let amount = get_attached_funds(&info.funds, &token)?;

            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: STAKING.load(deps.storage)?.to_string(),
                    msg: to_binary(&StakingExecuteMsg::DepositAndStake {
                        sender: Some(info.sender.to_string()),
                    })?,
                    funds: vec![Coin {
                        denom: token,
                        amount: amount.into(),
                    }],
                }))
                .add_attribute("action", "deposit_and_stake"))
        }
        ProxyExecuteMsg::Unstake { amount } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: STAKING.load(deps.storage)?.to_string(),
                msg: to_binary(&StakingExecuteMsg::Unstake {
                    amount,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "unstake")),
        ProxyExecuteMsg::Withdraw { amount } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: STAKING.load(deps.storage)?.to_string(),
                msg: to_binary(&StakingExecuteMsg::Withdraw {
                    amount,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "withdraw")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: ProxyQueryMsg) -> StdResult<Binary> {
    match msg.clone() {
        // Proxy
        ProxyQueryMsg::GetDataRequestsContract {} => {
            let contract = DATA_REQUESTS.load(deps.storage)?;
            let response: StdResult<GetContractResponse> = Ok(GetContractResponse {
                value: contract.to_string(),
            });
            to_binary(&response?)
        }
        ProxyQueryMsg::GetStakingContract {} => {
            let contract = DATA_REQUESTS.load(deps.storage)?;
            let response: StdResult<GetContractResponse> = Ok(GetContractResponse {
                value: contract.to_string(),
            });
            to_binary(&response?)
        }

        // DataRequests
        ProxyQueryMsg::GetDataRequest { dr_id: _dr_id } => {
            let query_response: GetDataRequestResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetDataRequestsFromPool {
            position: _position,
            limit: _limit,
        } => {
            let query_response: GetDataRequestsFromPoolResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetCommittedDataResult {
            dr_id: _dr_id,
            executor: _executor,
        } => {
            let query_response: GetCommittedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetCommittedDataResults { dr_id: _dr_id } => {
            let query_response: GetCommittedDataResultsResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetRevealedDataResult {
            dr_id: _dr_id,
            executor: _executor,
        } => {
            let query_response: GetRevealedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetRevealedDataResults { dr_id: _dr_id } => {
            let query_response: GetRevealedDataResultsResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::GetResolvedDataResult { dr_id: _dr_id } => {
            let query_response: GetResolvedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: DATA_REQUESTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }

        // Staking
        ProxyQueryMsg::GetDataRequestExecutor {
            executor: _executor,
        } => {
            let query_response: GetDataRequestExecutorResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: STAKING.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        ProxyQueryMsg::IsDataRequestExecutorEligible {
            executor: _executor,
        } => {
            let query_response: IsDataRequestExecutorEligibleResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: STAKING.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: ProxySudoMsg) -> Result<Response, ContractError> {
    match msg {
        ProxySudoMsg::SetDataRequests { contract } => {
            DATA_REQUESTS.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            Ok(Response::new().add_attribute("method", "set_data_requests"))
        }
        ProxySudoMsg::SetStaking { contract } => {
            STAKING.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            Ok(Response::new().add_attribute("method", "set_staking"))
        }
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    #[should_panic(expected = "ContractAlreadySet")]
    fn contract_already_set() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "token"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ProxyExecuteMsg::SetDataRequests {
            contract: "contract".to_string(),
        };
        let info = mock_info("creator", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ProxyExecuteMsg::SetDataRequests {
            contract: "contract2".to_string(),
        };
        let info = mock_info("creator", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    #[should_panic(expected = "NoFunds")]
    fn no_funds_provided() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "token"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ProxyExecuteMsg::SetDataRequests {
            contract: "contract".to_string(),
        };
        let info = mock_info("creator", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ProxyExecuteMsg::DepositAndStake;
        let info = mock_info("anyone", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
