use common::error::ContractError;
use cosmwasm_std::{Addr, Coin, DepsMut};

use crate::{
    consts::MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
    state::{ELIGIBLE_DATA_REQUEST_EXECUTORS, PROXY_CONTRACT},
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

pub fn get_attached_funds(funds: &[Coin], token: &str) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());
    amount.ok_or(ContractError::NoFunds)
}

pub fn validate_sender(
    deps: &DepsMut,
    caller: Addr,
    sender: Option<String>,
) -> Result<Addr, ContractError> {
    // if a sender is passed, caller must be the proxy contract
    match sender {
        Some(_sender) if caller != PROXY_CONTRACT.load(deps.storage)? => {
            Err(ContractError::NotProxy {})
        }
        Some(sender) => Ok(deps.api.addr_validate(&sender)?),
        None => Ok(caller),
    }
}
