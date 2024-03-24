use common::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::executors_registry::data_request_executors;
use crate::msg::StakingSudoMsg;
use crate::staking::staking;
use crate::state::{ADMIN, CONFIG, PENDING_ADMIN, PROXY_CONTRACT, TOKEN};
use common::consts::{
    INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, INITIAL_MINIMUM_STAKE_TO_REGISTER,
};
use common::msg::{
    GetAdminResponse, GetPendingOwnerResponse, GetStakingConfigResponse,
    StakingQueryMsg as QueryMsg,
};
use common::msg::{InstantiateMsg, StakingExecuteMsg as ExecuteMsg};
use common::state::StakingConfig;

use cosmwasm_std::StdResult;

// version info for migration info
const CONTRACT_NAME: &str = "staking";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    ADMIN.save(deps.storage, &deps.api.addr_validate(&msg.admin)?)?;
    PENDING_ADMIN.save(deps.storage, &None)?;
    let init_config = StakingConfig {
        minimum_stake_to_register: INITIAL_MINIMUM_STAKE_TO_REGISTER,
        minimum_stake_for_committee_eligibility: INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
    };
    CONFIG.save(deps.storage, &init_config)?;

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
        ExecuteMsg::TransferOwnership { new_admin } => {
            staking::transfer_ownership(deps, env, info, new_admin)
        }
        ExecuteMsg::AcceptOwnership {} => staking::accept_ownership(deps, env, info),
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
        QueryMsg::GetStakingConfig => to_binary(&GetStakingConfigResponse {
            value: CONFIG.load(deps.storage)?,
        }),
        QueryMsg::GetAdmin => to_binary(&GetAdminResponse {
            value: ADMIN.load(deps.storage)?,
        }),
        QueryMsg::GetPendingOwner => to_binary(&GetPendingOwnerResponse {
            value: PENDING_ADMIN.load(deps.storage)?,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: StakingSudoMsg) -> Result<Response, ContractError> {
    match msg {
        StakingSudoMsg::SetStakingConfig { config } => {
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_attribute("method", "set_staking_config"))
        }
    }
}

#[cfg(test)]
mod init_tests {
    use crate::helpers::{
        helper_accept_ownership, helper_get_admin, helper_get_pending_owner,
        helper_register_executor, helper_set_staking_config, helper_transfer_ownership,
        instantiate_staking_contract,
    };
    use common::error::ContractError;
    use common::state::StakingConfig;
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coins, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "token"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate_staking_contract(deps.as_mut(), info).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn only_proxy_can_pass_caller() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        // register a data request executor, while passing a sender
        let info = mock_info("anyone", &coins(2, "token"));

        let res = helper_register_executor(
            deps.as_mut(),
            info,
            Some("address".to_string()),
            Some("sender".to_string()),
        );
        assert_eq!(res.is_err_and(|x| x == ContractError::NotProxy), true);

        // register a data request executor from the proxy
        let info = mock_info("proxy", &coins(2, "token"));

        let _res = helper_register_executor(
            deps.as_mut(),
            info,
            Some("address".to_string()),
            Some("sender".to_string()),
        )
        .unwrap();
    }

    #[test]
    fn two_step_transfer_ownership() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        let admin = helper_get_admin(deps.as_mut()).value;
        assert_eq!(admin, "admin");

        let pending_admin = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_admin, None);

        // new-admin accepts ownership before admin calls transfer_ownership
        let info = mock_info("new-admin", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert_eq!(
            res.is_err_and(|x| x == ContractError::NoPendingAdminFound),
            true
        );

        // non-admin initiates transfering ownership
        let info = mock_info("non-admin", &coins(2, "token"));

        let res = helper_transfer_ownership(deps.as_mut(), info, "new_admin".to_string());
        assert_eq!(res.is_err_and(|x| x == ContractError::NotAdmin), true);

        // admin initiates transfering ownership
        let info = mock_info("admin", &coins(2, "token"));

        let res = helper_transfer_ownership(deps.as_mut(), info, "new_admin".to_string());
        assert!(res.is_ok());

        let admin = helper_get_admin(deps.as_mut()).value;
        assert_eq!(admin, "admin");

        let pending_admin = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_admin, Some(Addr::unchecked("new_admin")));

        // non-admin accepts ownership
        let info = mock_info("non-admin", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert_eq!(
            res.is_err_and(|x| x == ContractError::NotPendingOwner),
            true
        );

        // new admin accepts ownership
        let info = mock_info("new_admin", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert!(res.is_ok());

        let admin = helper_get_admin(deps.as_mut()).value;
        assert_eq!(admin, Addr::unchecked("new_admin"));

        let pending_admin = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_admin, None);
    }

    #[test]
    fn set_staking_config() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        let new_config = StakingConfig {
            minimum_stake_to_register: 100,
            minimum_stake_for_committee_eligibility: 200,
        };

        let _res = helper_set_staking_config(deps.as_mut(), new_config).unwrap();
    }
}
