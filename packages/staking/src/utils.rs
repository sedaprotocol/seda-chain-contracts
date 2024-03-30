use common::error::ContractError;
use cosmwasm_std::{Addr, Coin, DepsMut};

use crate::state::{CONFIG, ELIGIBLE_DATA_REQUEST_EXECUTORS, PROXY_CONTRACT};

pub fn apply_validator_eligibility(
    deps: DepsMut,
    sender: Addr,
    tokens_staked: u128,
) -> Result<(), ContractError> {
    if tokens_staked
        < CONFIG
            .load(deps.storage)?
            .minimum_stake_for_committee_eligibility
    {
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

pub fn caller_is_proxy(deps: &DepsMut, caller: Addr) -> Result<(), ContractError> {
    if caller != PROXY_CONTRACT.load(deps.storage)? {
        Err(ContractError::NotProxy {})
    } else {
        Ok(())
    }
}
