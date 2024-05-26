use cosmwasm_std::{Deps, DepsMut, StdResult};

use super::{
    state::{ALLOWLIST, CONFIG, STAKERS},
    Staker,
};
use crate::{error::ContractError, types::PublicKey};

/// Returns a staker with the given address, if it exists.
pub fn get_staker(deps: Deps, executor: PublicKey) -> StdResult<Option<Staker>> {
    let executor = STAKERS.may_load(deps.storage, &executor)?;
    Ok(executor)
}

// TODO: maybe move this to data-requests contract?
/// Returns whether an executor is eligible to participate in the committee.
pub fn is_executor_eligible(deps: Deps, executor: PublicKey) -> StdResult<bool> {
    let executor = STAKERS.may_load(deps.storage, &executor)?;
    let value = match executor {
        Some(staker) => staker.tokens_staked >= CONFIG.load(deps.storage)?.minimum_stake_for_committee_eligibility,
        None => false,
    };

    Ok(value)
}

pub fn is_staker_allowed(deps: &DepsMut, public_key: &PublicKey) -> Result<(), ContractError> {
    let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
    if allowlist_enabled {
        let is_allowed = ALLOWLIST.may_load(deps.storage, public_key)?;
        if is_allowed.is_none() {
            return Err(ContractError::NotOnAllowlist);
        }
    }

    Ok(())
}
