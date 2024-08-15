use owner::state::ALLOWLIST;
use seda_common::msgs::staking::Staker;

use super::{
    state::{CONFIG, STAKERS},
    *,
};

/// Returns a staker with the given address, if it exists.
pub fn get_staker(deps: Deps, executor: &PublicKey) -> StdResult<Option<Staker>> {
    let executor = STAKERS.may_load(deps.storage, executor)?;
    Ok(executor)
}

/// Returns whether an executor is eligible to participate in the committee.
pub fn is_executor_eligible(deps: Deps, executor: PublicKey) -> StdResult<bool> {
    let allowed = ALLOWLIST.may_load(deps.storage, &executor)?;
    // If the executor is not in the allowlist, they are not eligible.
    // If the executor is in the allowlist, but the value is false, they are not eligible.
    if allowed.is_none() || !allowed.unwrap() {
        return Ok(false);
    }

    let executor = STAKERS.may_load(deps.storage, &executor)?;
    Ok(match executor {
        Some(staker) => staker.tokens_staked >= CONFIG.load(deps.storage)?.minimum_stake_for_committee_eligibility,
        None => false,
    })
}
