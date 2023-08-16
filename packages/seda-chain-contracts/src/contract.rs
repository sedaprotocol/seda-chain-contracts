#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::data_request::data_requests;
use crate::error::ContractError;
use crate::executors_registry::data_request_executors;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::staking::staking;
use crate::state::{DATA_REQUESTS_COUNT, TOKEN, WASM_STORAGE_CONTRACT_ADDRESS};

use crate::data_request_result::data_request_results;
use cosmwasm_std::StdResult;

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
    WASM_STORAGE_CONTRACT_ADDRESS.save(deps.storage, &msg.wasm_storage_contract_address)?;
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
        ExecuteMsg::PostDataRequest {
            dr_id,
            value,
            nonce,
            chain_id,
            wasm_id,
            wasm_args
        } => data_requests::post_data_request(deps, info, dr_id, value, nonce, chain_id, wasm_id, wasm_args),
        ExecuteMsg::PostDataResult { dr_id, result } => {
            data_request_results::post_data_result(deps, info, dr_id, result)
        }
        ExecuteMsg::RegisterDataRequestExecutor { p2p_multi_address } => {
            data_request_executors::register_data_request_executor(deps, info, p2p_multi_address)
        }
        ExecuteMsg::UnregisterDataRequestExecutor {} => {
            data_request_executors::unregister_data_request_executor(deps, info)
        }
        ExecuteMsg::DepositAndStake => staking::deposit_and_stake(deps, env, info),
        ExecuteMsg::Unstake { amount } => staking::unstake(deps, env, info, amount),
        ExecuteMsg::Withdraw { amount } => staking::withdraw(deps, env, info, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequest { dr_id } => {
            to_binary(&data_requests::get_data_request(deps, dr_id)?)
        }
        QueryMsg::GetDataRequestsFromPool { position, limit } => to_binary(
            &data_requests::get_data_requests_from_pool(deps, position, limit)?,
        ),
        QueryMsg::GetDataResult { dr_id } => {
            to_binary(&data_request_results::get_data_result(deps, dr_id)?)
        }
        QueryMsg::GetDataRequestExecutor { executor } => to_binary(
            &data_request_executors::get_data_request_executor(deps, executor)?,
        ),
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            wasm_storage_contract_address: Addr::unchecked("wasm_storage_contract_address"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
