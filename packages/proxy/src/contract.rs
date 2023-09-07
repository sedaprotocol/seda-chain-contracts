#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::TOKEN;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use seda_chain_contracts::msg::ExecuteMsg as SedaChainContractsExecuteMsg;
use seda_chain_contracts::msg::QueryMsg;
use seda_chain_contracts::msg::{
    GetCommittedDataResultResponse, GetCommittedDataResultsResponse,
    GetDataRequestExecutorResponse, GetDataRequestResponse, GetDataRequestsFromPoolResponse,
    GetResolvedDataResultResponse, GetRevealedDataResultResponse, GetRevealedDataResultsResponse,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
    state::SEDA_CHAIN_CONTRACTS,
    utils::get_attached_funds,
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
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Admin
        ExecuteMsg::SetSedaChainContracts { contract } => {
            SEDA_CHAIN_CONTRACTS.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            Ok(Response::new().add_attribute("method", "set_seda_chain_contracts"))
        }

        // Delegated calls to contracts
        ExecuteMsg::PostDataRequest { posted_dr } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(&SedaChainContractsExecuteMsg::PostDataRequest { posted_dr })?,
                funds: vec![],
            }))
            .add_attribute("action", "post_data_request")),
        ExecuteMsg::CommitDataResult { dr_id, commitment } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(&SedaChainContractsExecuteMsg::CommitDataResult {
                    dr_id,
                    commitment,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "post_data_result")),
        ExecuteMsg::RevealDataResult { dr_id, reveal } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(&SedaChainContractsExecuteMsg::RevealDataResult {
                    dr_id,
                    reveal,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "post_data_result")),
        ExecuteMsg::RegisterDataRequestExecutor { p2p_multi_address } => {
            // require token deposit
            let token = TOKEN.load(deps.storage)?;
            let amount = get_attached_funds(&info.funds, &token)?;

            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::RegisterDataRequestExecutor {
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
        ExecuteMsg::UnregisterDataRequestExecutor {} => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(
                    &SedaChainContractsExecuteMsg::UnregisterDataRequestExecutor {
                        sender: Some(info.sender.to_string()),
                    },
                )?,
                funds: vec![],
            }))
            .add_attribute("action", "unregister_data_request_executor")),
        ExecuteMsg::DepositAndStake {} => {
            // require token deposit
            let token = TOKEN.load(deps.storage)?;
            let amount = get_attached_funds(&info.funds, &token)?;

            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::DepositAndStake {
                        sender: Some(info.sender.to_string()),
                    })?,
                    funds: vec![Coin {
                        denom: token,
                        amount: amount.into(),
                    }],
                }))
                .add_attribute("action", "deposit_and_stake"))
        }
        ExecuteMsg::Unstake { amount } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(&SedaChainContractsExecuteMsg::Unstake {
                    amount,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "unstake")),
        ExecuteMsg::Withdraw { amount } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                msg: to_binary(&SedaChainContractsExecuteMsg::Withdraw {
                    amount,
                    sender: Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "withdraw")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> cosmwasm_std::StdResult<Binary> {
    match msg.clone() {
        QueryMsg::GetDataRequest { dr_id: _dr_id } => {
            let query_response: GetDataRequestResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetDataRequestsFromPool {
            position: _position,
            limit: _limit,
        } => {
            let query_response: GetDataRequestsFromPoolResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetCommittedDataResult {
            dr_id: _dr_id,
            executor: _executor,
        } => {
            let query_response: GetCommittedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetCommittedDataResults { dr_id: _dr_id } => {
            let query_response: GetCommittedDataResultsResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetRevealedDataResult {
            dr_id: _dr_id,
            executor: _executor,
        } => {
            let query_response: GetRevealedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetRevealedDataResults { dr_id: _dr_id } => {
            let query_response: GetRevealedDataResultsResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetResolvedDataResult { dr_id: _dr_id } => {
            let query_response: GetResolvedDataResultResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
        QueryMsg::GetDataRequestExecutor {
            executor: _executor,
        } => {
            let query_response: GetDataRequestExecutorResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&msg)?,
                }))?;
            Ok(to_binary(&query_response)?)
        }
    }
}
