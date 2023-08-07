use cosmwasm_std::{Addr, DepsMut};

use crate::{
    consts::MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY, state::ELIGIBLE_DATA_REQUEST_EXECUTORS,
    ContractError,
};

pub fn apply_validator_eligibility(
    deps: DepsMut,
    sender: Addr,
    tokens_staked: u128,
) -> Result<(), ContractError> {
    if tokens_staked < MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY {
        if ELIGIBLE_DATA_REQUEST_EXECUTORS.has(deps.storage, sender.clone()) {
            ELIGIBLE_DATA_REQUEST_EXECUTORS.remove(deps.storage, sender);
        }
    } else {
        ELIGIBLE_DATA_REQUEST_EXECUTORS.save(deps.storage, sender, &true)?;
    }
    Ok(())
}
