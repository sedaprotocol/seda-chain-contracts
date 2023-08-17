use cosmwasm_std::{Addr, Coin, DepsMut};
use sha3::{Digest, Keccak256};

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

pub fn check_eligibility(deps: &DepsMut, dr_executor: Addr) -> Result<bool, ContractError> {
    let is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS.load(deps.storage, dr_executor)?;
    Ok(is_eligible)
}

pub fn get_attached_funds(funds: &[Coin], token: String) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());
    amount.ok_or(ContractError::NoFunds)
}

pub fn pad_to_32_bytes(value: u128) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let small_bytes = &value.to_be_bytes();
    bytes[(32 - small_bytes.len())..].copy_from_slice(small_bytes);
    bytes
}

pub fn hash_update(hasher: &mut Keccak256, value: u128) {
    let bytes = pad_to_32_bytes(value);
    hasher.update(bytes);
}
