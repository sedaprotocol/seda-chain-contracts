use cosmwasm_std::Uint128;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use seda_common::msgs::*;
use staking::StakingConfig;

use crate::{
    consts::{INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, INITIAL_MINIMUM_STAKE_TO_REGISTER},
    error::ContractError,
    msgs::{
        owner::state::{OWNER, PENDING_OWNER},
        staking::state::CONFIG,
        ExecuteHandler,
        QueryHandler,
        SudoMsg,
    },
    state::{CHAIN_ID, TOKEN},
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
    CHAIN_ID.save(deps.storage, &msg.chain_id)?;
    PENDING_OWNER.save(deps.storage, &None)?;
    let init_config = StakingConfig {
        minimum_stake_to_register:               Uint128::new(INITIAL_MINIMUM_STAKE_TO_REGISTER),
        minimum_stake_for_committee_eligibility: Uint128::new(INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY),
        allowlist_enabled:                       false,
    };
    CONFIG.save(deps.storage, &init_config)?;
    crate::msgs::data_requests::state::init_data_requests(deps.storage)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    msg.execute(deps, env, info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, sudo: SudoMsg) -> Result<Response, ContractError> {
    sudo.execute(deps, env)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.query(deps, env)
}
