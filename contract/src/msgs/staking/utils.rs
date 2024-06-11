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
