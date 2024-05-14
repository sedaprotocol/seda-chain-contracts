use common::{
    consts::{INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, INITIAL_MINIMUM_STAKE_TO_REGISTER},
    error::ContractError,
    msg::{
        GetOwnerResponse,
        GetPendingOwnerResponse,
        GetStakingConfigResponse,
        InstantiateMsg,
        StakingExecuteMsg as ExecuteMsg,
        StakingQueryMsg as QueryMsg,
    },
    state::StakingConfig,
};
use cosmwasm_std::StdResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    config,
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
    // PROXY_CONTRACT.save(deps.storage, &deps.api.addr_validate(&msg.proxy)?)?;
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
        ExecuteMsg::RegisterAndStake { signature, memo } => staking::register_and_stake(deps, info, signature, memo),
        ExecuteMsg::Unregister { signature } => staking::unregister(deps, info, signature),
        ExecuteMsg::IncreaseStake { signature } => staking::increase_stake(deps, env, info, signature),
        ExecuteMsg::Unstake { signature, amount } => staking::unstake(deps, env, info, signature, amount),
        ExecuteMsg::Withdraw { signature, amount } => staking::withdraw(deps, env, info, signature, amount),
        ExecuteMsg::TransferOwnership { new_owner } => config::transfer_ownership(deps, env, info, new_owner),
        ExecuteMsg::AcceptOwnership {} => config::accept_ownership(deps, env, info),
        ExecuteMsg::SetStakingConfig { config } => config::set_staking_config(deps, env, info, config),
        ExecuteMsg::AddToAllowlist { pub_key } => config::add_to_allowlist(deps, info, pub_key),
        ExecuteMsg::RemoveFromAllowlist { pub_key } => config::remove_from_allowlist(deps, info, pub_key),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStaker { executor } => to_json_binary(&staking::get_staker(deps, executor)?),
        QueryMsg::IsDataRequestExecutorEligible { executor } => {
            to_json_binary(&staking::is_data_request_executor_eligible(deps, executor)?)
        }
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
