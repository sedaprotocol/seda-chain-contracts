use cosmwasm_std::Coin;

use crate::error::ContractError;

pub fn get_attached_funds(funds: &[Coin], token: &str) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());

    amount.ok_or(ContractError::NoFunds)
}
