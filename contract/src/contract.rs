use cosmwasm_std::Event;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use data_requests::TimeoutConfig;
use seda_common::msgs::*;
use staking::StakingConfig;

use crate::{
    consts::{
        INITIAL_COMMIT_TIMEOUT_IN_BLOCKS,
        INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
        INITIAL_MINIMUM_STAKE_TO_REGISTER,
        INITIAL_REVEAL_TIMEOUT_IN_BLOCKS,
    },
    error::ContractError,
    msgs::{
        data_requests::{execute::dr_events::create_timeout_config_event, state::TIMEOUT_CONFIG},
        owner::state::{OWNER, PENDING_OWNER},
        staking::{
            execute::staking_events::create_staking_config_event,
            state::{STAKERS, STAKING_CONFIG},
        },
        ExecuteHandler,
        QueryHandler,
        SudoHandler,
    },
    state::{CHAIN_ID, PAUSED, TOKEN},
};

// version info for migration info
const CONTRACT_NAME: &str = "staking";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_REVISION: &str = env!("GIT_REVISION");

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
    PAUSED.save(deps.storage, &false)?;

    let init_staking_config = msg.staking_config.unwrap_or(StakingConfig {
        minimum_stake_to_register:               INITIAL_MINIMUM_STAKE_TO_REGISTER,
        minimum_stake_for_committee_eligibility: INITIAL_MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
        allowlist_enabled:                       false,
    });
    STAKING_CONFIG.save(deps.storage, &init_staking_config)?;

    let init_timeout_config = msg.timeout_config.unwrap_or(TimeoutConfig {
        commit_timeout_in_blocks: INITIAL_COMMIT_TIMEOUT_IN_BLOCKS,
        reveal_timeout_in_blocks: INITIAL_REVEAL_TIMEOUT_IN_BLOCKS,
    });
    TIMEOUT_CONFIG.save(deps.storage, &init_timeout_config)?;

    STAKERS.initialize(deps.storage)?;
    crate::msgs::data_requests::state::init_data_requests(deps.storage)?;

    Ok(Response::new().add_attribute("method", "instantiate").add_events([
        Event::new("seda-contract").add_attributes([
            ("action", "instantiate".to_string()),
            ("version", CONTRACT_VERSION.to_string()),
            ("chain_id", msg.chain_id),
            ("owner", msg.owner),
            ("token", msg.token),
            ("git_revision", GIT_REVISION.to_string()),
        ]),
        create_staking_config_event(init_staking_config),
        create_timeout_config_event(init_timeout_config),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    msg.execute(deps, env, info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, sudo: SudoMsg) -> Result<Response, ContractError> {
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::ContractPaused("sudo messages".to_string()));
    }

    sudo.sudo(deps, env)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.query(deps, env)
}
