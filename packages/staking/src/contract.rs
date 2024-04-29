use crate::allowlist::allow_list;
use crate::config::config;
use crate::executors_registry::data_request_executors;
use crate::staking::staking;
use crate::state::{CONFIG, OWNER, PENDING_OWNER, PROXY_CONTRACT, TOKEN};
use common::consts::{
    INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, INITIAL_MINIMUM_STAKE_TO_REGISTER,
};
use common::error::ContractError;
use common::msg::{
    GetOwnerResponse, GetPendingOwnerResponse, GetStakingConfigResponse,
    StakingQueryMsg as QueryMsg,
};
use common::msg::{InstantiateMsg, StakingExecuteMsg as ExecuteMsg};
use common::state::StakingConfig;
use cosmwasm_std::StdResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use cw2::set_contract_version;

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
    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    PENDING_OWNER.save(deps.storage, &None)?;
    let init_config = StakingConfig {
        minimum_stake_to_register: INITIAL_MINIMUM_STAKE_TO_REGISTER,
        minimum_stake_for_committee_eligibility: INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
        allowlist_enabled: false,
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
            public_key,
            signature,
            memo,
            sender,
        } => data_request_executors::register_data_request_executor(
            deps, info, public_key, signature, memo, sender,
        ),
        ExecuteMsg::UnregisterDataRequestExecutor {
            public_key,
            signature,
            sender,
        } => data_request_executors::unregister_data_request_executor(
            deps, info, public_key, signature, sender,
        ),
        ExecuteMsg::DepositAndStake {
            public_key,
            signature,
            sender,
        } => staking::deposit_and_stake(deps, env, info, public_key, signature, sender),
        ExecuteMsg::Unstake {
            public_key,
            signature,
            amount,
            sender,
        } => staking::unstake(deps, env, info, public_key, signature, amount, sender),
        ExecuteMsg::Withdraw {
            public_key,
            signature,
            amount,
            sender,
        } => staking::withdraw(deps, env, info, public_key, signature, amount, sender),
        ExecuteMsg::TransferOwnership { new_owner } => {
            config::transfer_ownership(deps, env, info, new_owner)
        }
        ExecuteMsg::AcceptOwnership {} => config::accept_ownership(deps, env, info),
        ExecuteMsg::SetStakingConfig { config } => {
            config::set_staking_config(deps, env, info, config)
        }
        ExecuteMsg::AddToAllowlist { address, sender } => {
            allow_list::add_to_allowlist(deps, info, sender, address)
        }
        ExecuteMsg::RemoveFromAllowlist { address, sender } => {
            allow_list::remove_from_allowlist(deps, info, sender, address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequestExecutor { executor } => to_json_binary(
            &data_request_executors::get_data_request_executor(deps, executor)?,
        ),
        QueryMsg::IsDataRequestExecutorEligible { executor } => to_json_binary(
            &data_request_executors::is_data_request_executor_eligible(deps, executor)?,
        ),
        QueryMsg::GetStakingConfig => to_json_binary(&GetStakingConfigResponse {
            value: CONFIG.load(deps.storage)?,
        }),
        QueryMsg::GetOwner => to_json_binary(&GetOwnerResponse {
            value: OWNER.load(deps.storage)?,
        }),
        QueryMsg::GetPendingOwner => to_json_binary(&GetPendingOwnerResponse {
            value: PENDING_OWNER.load(deps.storage)?,
        }),
    }
}

#[cfg(test)]
mod init_tests {
    use crate::helpers::{
        helper_accept_ownership, helper_get_owner, helper_get_pending_owner,
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
            vec![0; 33],
            vec![0; 33],
            None,
            Some("sender".to_string()),
        );
        assert_eq!(res.is_err_and(|x| x == ContractError::NotProxy), true);

        // register a data request executor from the proxy
        let info = mock_info("proxy", &coins(2, "token"));

        let _res = helper_register_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            vec![0; 33],
            None,
            Some("sender".to_string()),
        )
        .unwrap();
    }

    #[test]
    fn two_step_transfer_ownership() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        let owner = helper_get_owner(deps.as_mut()).value;
        assert_eq!(owner, "owner");

        let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_owner, None);

        // new-owner accepts ownership before owner calls transfer_ownership
        let info = mock_info("new-owner", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert_eq!(
            res.is_err_and(|x| x == ContractError::NoPendingOwnerFound),
            true
        );

        // non-owner initiates transfering ownership
        let info = mock_info("non-owner", &coins(2, "token"));

        let res = helper_transfer_ownership(deps.as_mut(), info, "new_owner".to_string());
        assert_eq!(res.is_err_and(|x| x == ContractError::NotOwner), true);

        // owner initiates transfering ownership
        let info = mock_info("owner", &coins(2, "token"));

        let res = helper_transfer_ownership(deps.as_mut(), info, "new_owner".to_string());
        assert!(res.is_ok());

        let owner = helper_get_owner(deps.as_mut()).value;
        assert_eq!(owner, "owner");

        let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_owner, Some(Addr::unchecked("new_owner")));

        // non-owner accepts ownership
        let info = mock_info("non-owner", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert_eq!(
            res.is_err_and(|x| x == ContractError::NotPendingOwner),
            true
        );

        // new owner accepts ownership
        let info = mock_info("new_owner", &coins(2, "token"));

        let res = helper_accept_ownership(deps.as_mut(), info);
        assert!(res.is_ok());

        let owner = helper_get_owner(deps.as_mut()).value;
        assert_eq!(owner, Addr::unchecked("new_owner"));

        let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
        assert_eq!(pending_owner, None);
    }

    #[test]
    fn set_staking_config() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info.clone()).unwrap();

        let new_config = StakingConfig {
            minimum_stake_to_register: 100,
            minimum_stake_for_committee_eligibility: 200,
            allowlist_enabled: false,
        };

        // non-owner sets staking config
        let info = mock_info("non-owner", &coins(0, "token"));

        let res = helper_set_staking_config(deps.as_mut(), info, new_config.clone());
        assert_eq!(res.is_err_and(|x| x == ContractError::NotOwner), true);

        // owner sets staking config
        let info = mock_info("owner", &coins(0, "token"));

        let res = helper_set_staking_config(deps.as_mut(), info, new_config);
        assert!(res.is_ok());
    }
}
