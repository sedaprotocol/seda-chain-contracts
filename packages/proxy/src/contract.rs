#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use seda_chain_contracts::msg::ExecuteMsg as SedaChainContractsExecuteMsg;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::SEDA_CHAIN_CONTRACTS,
};

// version info
const CONTRACT_NAME: &str = "seda-bin-storage";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::PostDataRequest { args } => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::PostDataRequest { args })?,
                    funds: vec![],
                }))
                .add_attribute("action", "post_data_request"))
        }
        ExecuteMsg::PostDataResult { dr_id, result } => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::PostDataResult { dr_id, result })?,
                    funds: vec![],
                }))
                .add_attribute("action", "post_data_result"))
        }
        ExecuteMsg::RegisterDataRequestExecutor { p2p_multi_address } => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::RegisterDataRequestExecutor {
                        p2p_multi_address,
                    })?,
                    funds: vec![],
                }))
                .add_attribute("action", "register_data_request_executor"))
        }
        ExecuteMsg::UnregisterDataRequestExecutor {} => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::UnregisterDataRequestExecutor {})?,
                    funds: vec![],
                }))
                .add_attribute("action", "unregister_data_request_executor"))
        }
        // TODO: forward funds
        ExecuteMsg::DepositAndStake {} => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::DepositAndStake {})?,
                    funds: vec![],
                }))
                .add_attribute("action", "deposit_and_stake"))
        }
        ExecuteMsg::Unstake { amount } => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::Unstake { amount })?,
                    funds: vec![],
                }))
                .add_attribute("action", "unstake"))
        }
        ExecuteMsg::Withdraw { amount } => {
            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SEDA_CHAIN_CONTRACTS.load(deps.storage)?.to_string(),
                    msg: to_binary(&SedaChainContractsExecuteMsg::Withdraw { amount })?,
                    funds: vec![],
                }))
                .add_attribute("action", "withdraw"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> cosmwasm_std::StdResult<Binary> {
    match msg {
        // QueryMsg::QueryEntry { key } => cosmwasm_std::to_binary(&query_binary(deps, &key)?),
    }
}
