#[allow(clippy::module_inception)]
pub mod config {
    use common::{error::ContractError, state::StakingConfig};
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};

    use crate::{
        contract::CONTRACT_VERSION,
        state::{CONFIG, OWNER, PENDING_OWNER},
    };

    /// Transfer contract ownership to a new owner
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
                ("new_owner", &new_owner),
            ])]))
    }

    /// Accept contract ownership
    pub fn accept_ownership(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
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
            .add_events([Event::new("seda-accept-ownership").add_attributes([
                ("version", CONTRACT_VERSION),
                ("new_owner", info.sender.as_ref()),
            ])]))
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
            ])]))
    }
}