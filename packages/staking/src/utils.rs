use common::{error::ContractError, types::Secpk256k1PublicKey};
use cosmwasm_std::{Coin, DepsMut};

use crate::state::{ALLOWLIST, CONFIG, ELIGIBLE_DATA_REQUEST_EXECUTORS};

pub fn update_dr_elig(
    deps: DepsMut,
    public_key: &Secpk256k1PublicKey,
    tokens_staked: u128,
) -> Result<(), ContractError> {
    if tokens_staked < CONFIG.load(deps.storage)?.minimum_stake_for_committee_eligibility {
        if ELIGIBLE_DATA_REQUEST_EXECUTORS.has(deps.storage, public_key) {
            ELIGIBLE_DATA_REQUEST_EXECUTORS.remove(deps.storage, public_key);
        }
    } else {
        ELIGIBLE_DATA_REQUEST_EXECUTORS.save(deps.storage, public_key, &true)?;
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

pub fn if_allowlist_enabled(deps: &DepsMut, public_key: &Secpk256k1PublicKey) -> Result<(), ContractError> {
    let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
    if allowlist_enabled {
        let is_allowed = ALLOWLIST.may_load(deps.storage, public_key)?;
        if is_allowed.is_none() {
            return Err(ContractError::NotOnAllowlist);
        }
    }
    Ok(())
}
