use cosmwasm_std::{Coin, Uint128};

use crate::error::ContractError;

pub fn get_attached_funds(funds: &[Coin], token: &str) -> Result<Uint128, ContractError> {
    let amount = funds.iter().find(|coin| coin.denom == token).map(|coin| coin.amount);

    amount.ok_or(ContractError::NoFunds)
}
