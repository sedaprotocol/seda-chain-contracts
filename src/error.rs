use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("No funds provided")]
    NoFunds,
    #[error("Insufficient funds. Required: {0}, available: {1}")]
    InsufficientFunds(u128, u128),
    #[error("Executor has staked tokens or tokens pending withdrawal")]
    ExecutorHasTokens,
}
