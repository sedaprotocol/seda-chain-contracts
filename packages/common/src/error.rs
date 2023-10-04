use crate::types::Hash;
use cosmwasm_std::StdError;
use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    // staking contract errors
    #[error("No funds provided")]
    NoFunds,
    #[error("Executor has staked tokens or tokens pending withdrawal")]
    ExecutorHasTokens,

    // DR contract errors
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Insufficient funds. Required: {0}, available: {1}")]
    InsufficientFunds(u128, u128),
    #[error("Invalid data request id, expected: {0}, actual: {1}")]
    InvalidDataRequestId(Hash, Hash),
    #[error("Data request already exists")]
    DataRequestAlreadyExists,
    #[error("Caller is not an eligible data request executor")]
    IneligibleExecutor,
    #[error("Caller has already committed on this data request")]
    AlreadyCommitted,
    #[error("Reveal stage has not started yet")]
    RevealNotStarted,
    #[error("Executor has not committed on this data request")]
    NotCommitted,
    #[error("Executor has already revealed on this data request")]
    AlreadyRevealed,
    #[error("Revealed result does not match the committed result")]
    RevealMismatch,
    #[error("Only proxy can pass a sender")]
    NotProxy,
    #[error("Arg cannot be empty: {0}")]
    EmptyArg(String),

    // proxy errors
    #[error("Contract already set")]
    ContractAlreadySet,
}
