use cosmwasm_std::StdResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    config,
    consts::{INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, INITIAL_MINIMUM_STAKE_TO_REGISTER},
    error::ContractError,
    msgs::{
        staking::StakingConfig,
        ExecuteMsg,
        InstantiateMsg,
        OwnerExecuteMsg,
        OwnerQueryMsg,
        QueryMsg,
        StakingExecuteMsg,
        StakingQueryMsg,
    },
    staking,
    state::{CONFIG, OWNER, PENDING_OWNER, TOKEN},
};

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
    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    PENDING_OWNER.save(deps.storage, &None)?;
    let init_config = StakingConfig {
        minimum_stake_to_register:               INITIAL_MINIMUM_STAKE_TO_REGISTER,
        minimum_stake_for_committee_eligibility: INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
        allowlist_enabled:                       false,
    };
    CONFIG.save(deps.storage, &init_config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Staking(msg) => match msg {
            StakingExecuteMsg::RegisterAndStake { signature, memo } => {
                staking::register_and_stake(deps, info, signature, memo)
            }
            StakingExecuteMsg::IncreaseStake { signature } => staking::increase_stake(deps, env, info, signature),
            StakingExecuteMsg::Unstake { signature, amount } => staking::unstake(deps, env, info, signature, amount),
            StakingExecuteMsg::Withdraw { signature, amount } => staking::withdraw(deps, env, info, signature, amount),
            StakingExecuteMsg::Unregister { signature } => staking::unregister(deps, info, signature),
            StakingExecuteMsg::SetStakingConfig { config } => config::set_staking_config(deps, env, info, config),
        },
        ExecuteMsg::Owner(msg) => match msg {
            OwnerExecuteMsg::TransferOwnership { new_owner } => config::transfer_ownership(deps, env, info, new_owner),
            OwnerExecuteMsg::AcceptOwnership {} => config::accept_ownership(deps, env, info),
            OwnerExecuteMsg::AddToAllowlist { pub_key } => config::add_to_allowlist(deps, info, pub_key),
            OwnerExecuteMsg::RemoveFromAllowlist { pub_key } => config::remove_from_allowlist(deps, info, pub_key),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Staking(msg) => match msg {
            StakingQueryMsg::GetStaker { executor } => to_json_binary(&staking::get_staker(deps, executor)?),
            StakingQueryMsg::IsExecutorEligible { executor } => {
                to_json_binary(&staking::is_executor_eligible(deps, executor)?)
            }
            StakingQueryMsg::GetStakingConfig => to_json_binary(&CONFIG.load(deps.storage)?),
        },
        QueryMsg::Owner(msg) => match msg {
            OwnerQueryMsg::GetOwner => to_json_binary(&OWNER.load(deps.storage)?),

            OwnerQueryMsg::GetPendingOwner => to_json_binary(&PENDING_OWNER.load(deps.storage)?),
        },
    }
}
