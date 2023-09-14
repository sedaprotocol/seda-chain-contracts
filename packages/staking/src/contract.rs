#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::executors_registry::data_request_executors;
use crate::msg::InstantiateMsg;
use crate::staking::staking;
use crate::state::{PROXY_CONTRACT, TOKEN};
use common::msg::StakingExecuteMsg as ExecuteMsg;
use common::msg::StakingQueryMsg as QueryMsg;

use cosmwasm_std::StdResult;

// version info for migration info
const CONTRACT_NAME: &str = "staking";
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
    PROXY_CONTRACT.save(deps.storage, &deps.api.addr_validate(&msg.proxy)?)?;
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
        ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address,
            sender,
        } => data_request_executors::register_data_request_executor(
            deps,
            info,
            p2p_multi_address,
            sender,
        ),
        ExecuteMsg::UnregisterDataRequestExecutor { sender } => {
            data_request_executors::unregister_data_request_executor(deps, info, sender)
        }
        ExecuteMsg::DepositAndStake { sender } => {
            staking::deposit_and_stake(deps, env, info, sender)
        }
        ExecuteMsg::Unstake { amount, sender } => staking::unstake(deps, env, info, amount, sender),
        ExecuteMsg::Withdraw { amount, sender } => {
            staking::withdraw(deps, env, info, amount, sender)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequestExecutor { executor } => to_binary(
            &data_request_executors::get_data_request_executor(deps, executor)?,
        ),
        QueryMsg::IsDataRequestExecutorEligible { executor } => to_binary(
            &data_request_executors::is_data_request_executor_eligible(deps, executor)?,
        ),
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "token"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn only_proxy_can_pass_caller() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // register a data request executor, while passing a sender
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: Some("sender".to_string()),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        // register a data request executor from the proxy
        let info = mock_info("proxy", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: Some("sender".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
