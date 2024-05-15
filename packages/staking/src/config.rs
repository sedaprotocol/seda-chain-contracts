use common::{error::ContractError, state::StakingConfig, types::Secp256k1PublicKey};
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};

use crate::{
    contract::CONTRACT_VERSION,
    state::{ALLOWLIST, CONFIG, OWNER, PENDING_OWNER},
};

/// Start 2-step process for transfer contract ownership to a new address
pub fn transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::NotOwner);
    }
    PENDING_OWNER.save(deps.storage, &Some(deps.api.addr_validate(&new_owner)?))?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_events([Event::new("seda-transfer-ownership").add_attributes([
            ("version", CONTRACT_VERSION),
            ("sender", info.sender.as_ref()),
            ("pending_owner", &new_owner),
        ])]))
}

/// Accept transfer contract ownership (previously triggered by owner)
pub fn accept_ownership(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let pending_owner = PENDING_OWNER.load(deps.storage)?;
    if pending_owner.is_none() {
        return Err(ContractError::NoPendingOwnerFound);
    }
    if pending_owner.is_some_and(|owner| owner != info.sender) {
        return Err(ContractError::NotPendingOwner);
    }
    OWNER.save(deps.storage, &info.sender)?;
    PENDING_OWNER.save(deps.storage, &None)?;

    Ok(Response::new()
        .add_attribute("action", "accept-ownership")
        .add_events([Event::new("seda-accept-ownership")
            .add_attributes([("version", CONTRACT_VERSION), ("new_owner", info.sender.as_ref())])]))
}

/// Set staking config
pub fn set_staking_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    config: StakingConfig,
) -> Result<Response, ContractError> {
    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::NotOwner);
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "set-staking-config")
        .add_events([Event::new("set-staking-config").add_attributes([
            ("version", CONTRACT_VERSION),
            (
                "minimum_stake_for_committee_eligibility",
                &config.minimum_stake_for_committee_eligibility.to_string(),
            ),
            (
                "minimum_stake_to_register",
                &config.minimum_stake_to_register.to_string(),
            ),
            ("allowlist_enabled", &config.allowlist_enabled.to_string()),
        ])]))
}

/// Add a `Secp256k1PublicKey` to the allow list
pub fn add_to_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    // require the sender to be the OWNER
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::NotOwner);
    }

    // add the address to the allowlist
    ALLOWLIST.save(deps.storage, pub_key.as_ref(), &true)?;

    Ok(Response::new().add_attribute("action", "add-to-allowlist").add_event(
        Event::new("add-to-allowlist")
            .add_attributes([("version", CONTRACT_VERSION), ("pub_key", &hex::encode(pub_key))]),
    ))
}

/// Remove a `Secp256k1PublicKey` to the allow list
pub fn remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    // require the sender to be the OWNER
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::NotOwner);
    }

    // remove the address from the allowlist
    ALLOWLIST.remove(deps.storage, pub_key.as_ref());

    Ok(Response::new()
        .add_attribute("action", "remove-from-allowlist")
        .add_event(
            Event::new("remove-from-allowlist")
                .add_attributes([("version", CONTRACT_VERSION), ("pub_key", &hex::encode(pub_key))]),
        ))
}
