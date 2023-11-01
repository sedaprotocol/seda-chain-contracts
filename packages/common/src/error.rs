use crate::types::Hash;
use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    // staking contract errors
    #[error("NoFunds: No funds provided")]
    NoFunds,
    #[error("ExecutorHasTokens: Executor has staked tokens or tokens pending withdrawal")]
    ExecutorHasTokens,

    // DR contract errors
    #[error("InsufficientFunds: Insufficient funds. Required: {0}, available: {1}")]
    InsufficientFunds(u128, u128),
    #[error("InvalidDataRequestId: Invalid data request id, expected: {0:#?}, actual: {1:#?}")]
    InvalidDataRequestId(Hash, Hash),
    #[error("DataRequestAlreadyExists: Data request already exists")]
    DataRequestAlreadyExists,
    #[error("IneligibleExecutor: Caller is not an eligible data request executor")]
    IneligibleExecutor,
    #[error("AlreadyCommitted: Caller has already committed on this data request")]
    AlreadyCommitted,
    #[error("RevealNotStarted: Reveal stage has not started yet")]
    RevealNotStarted,
    #[error("NotCommitted: Executor has not committed on this data request")]
    NotCommitted,
    #[error("AlreadyRevealed: Executor has already revealed on this data request")]
    AlreadyRevealed,
    #[error("RevealMismatch: Revealed result does not match the committed result")]
    RevealMismatch,
    #[error("NotProxy: Only proxy can pass a sender")]
    NotProxy,
    #[error("EmptyArg: Arg cannot be empty: {0}")]
    EmptyArg(String),

    // proxy errors
    #[error("ContractAlreadySet: Contract already set")]
    ContractAlreadySet,
    #[error("NotContractCreator: Caller must be the contract creator")]
    NotContractCreator,
    #[error("UnknownReplyId: Unknown reply ID: {0}")]
    UnknownReplyId(String),
    #[error("UnexpectedError: Unexpected error: {0}")]
    UnexpectedError(String),
    #[error("ParseReplyError: Parse reply error: {0}")]
    ParseReplyError(#[from] ParseReplyError),
    #[error("FromHex: Invalid hexadecimal input: {0}")]
    FromHex(FromHexError),
}

impl From<FromHexError> for ContractError {
    fn from(err: FromHexError) -> Self {
        ContractError::FromHex(err)
    }
}
