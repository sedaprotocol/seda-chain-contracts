use cosmwasm_std::{Addr, Coin, DepsMut};
use sha3::{Digest, Keccak256};

use crate::{
    consts::MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY,
    state::{DataRequest, DataRequestInputs, ELIGIBLE_DATA_REQUEST_EXECUTORS, PROXY_CONTRACT},
    types::Bytes,
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

pub fn hash_data_request(posted_dr: DataRequestInputs) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(posted_dr.dr_binary_id);
    hasher.update(posted_dr.dr_inputs);
    hasher.update(posted_dr.gas_limit.to_be_bytes());
    hasher.update(posted_dr.gas_price.to_be_bytes());
    hasher.update(posted_dr.memo);
    hasher.update(posted_dr.payback_address);
    hasher.update(posted_dr.replication_factor.to_be_bytes());
    hasher.update(posted_dr.seda_payload);
    hasher.update(posted_dr.tally_binary_id);
    hasher.update(posted_dr.tally_inputs);

    format!("0x{}", hex::encode(hasher.finalize()))
}

pub fn hash_data_result(
    dr: &DataRequest,
    block_height: u64,
    exit_code: u8,
    result: &Bytes,
) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(dr.dr_id.as_bytes());
    hasher.update(block_height.to_be_bytes());
    hasher.update(exit_code.to_be_bytes());
    hasher.update(result);
    hasher.update(dr.payback_address.clone());
    hasher.update(dr.seda_payload.clone());
    format!("0x{}", hex::encode(hasher.finalize()))
}

pub fn validate_sender(
    deps: &DepsMut,
    caller: Addr,
    sender: Option<String>,
) -> Result<Addr, ContractError> {
    match sender {
        Some(sender) => {
            // if a sender is passed, caller must be the proxy contract
            if caller != PROXY_CONTRACT.load(deps.storage)? {
                return Err(ContractError::NotProxy {});
            }
            Ok(deps.api.addr_validate(&sender)?)
        }
        None => Ok(caller),
    }
}
