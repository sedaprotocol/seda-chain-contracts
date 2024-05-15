use common::{error::ContractError, types::Secpk256k1PublicKey};
use cosmwasm_std::{Coin, DepsMut};

use crate::state::{ALLOWLIST, CONFIG};

pub fn get_attached_funds(funds: &[Coin], token: &str) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());

    amount.ok_or(ContractError::NoFunds)
}

pub fn is_staker_allowed(deps: &DepsMut, public_key: &Secpk256k1PublicKey) -> Result<(), ContractError> {
    let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
    if allowlist_enabled {
        let is_allowed = ALLOWLIST.may_load(deps.storage, public_key)?;
        if is_allowed.is_none() {
            return Err(ContractError::NotOnAllowlist);
        }
    }

    Ok(())
}
