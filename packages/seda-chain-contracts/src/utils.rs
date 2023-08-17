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

pub fn get_attached_funds(funds: &[Coin], token: &str) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());
    amount.ok_or(ContractError::NoFunds)
}

pub fn pad_to_32_bytes(value: &u128) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let small_bytes = &value.to_be_bytes();
    bytes[(32 - small_bytes.len())..].copy_from_slice(small_bytes);
    bytes
}

pub fn hash_update(hasher: &mut Keccak256, value: &u128) {
    let bytes = pad_to_32_bytes(value);
    hasher.update(bytes);
}

pub fn hash_data_request(
    nonce: &u128,
    value: &str,
    chain_id: &u128,
    wasm_id: &[u8],
    wasm_args: Vec<Vec<u8>>,
) -> String {
    let mut hasher = Keccak256::new();
    hash_update(&mut hasher, nonce);
    hasher.update(value.as_bytes());
    hash_update(&mut hasher, chain_id);
    hasher.update(wasm_id);
    for arg in wasm_args.iter() {
        hasher.update(arg.as_slice());
    }
    let hash_bytes = hasher.finalize();
    format!("0x{}", hex::encode(hash_bytes))
}
