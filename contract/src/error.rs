use cosmwasm_std::{StdError, Uint128};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub enum ContractError {
    #[cfg(not(test))]
    #[error(transparent)]
    Std(#[from] StdError),

    #[cfg(test)]
    #[error("{0}")]
    Std(String),

    #[cfg(test)]
    #[error("{0}")]
    Dbg(String),

    // staking contract errors
    #[error("NoFunds: No funds provided")]
    NoFunds,
    #[error("ExecutorHasTokens: Executor has staked tokens or tokens pending withdrawal")]
    ExecutorHasTokens,
    #[error("NotOwner: Only owner can transfer ownership")]
    NotOwner,
    #[error("NotPendingOwner: Only pending owner can accept ownership")]
    NotPendingOwner,
    #[error("NoPendingOwnerFound: No pending owner found")]
    NoPendingOwnerFound,
    #[error("NotOnAllowlist: Address is not on the allowlist")]
    NotOnAllowlist,
    #[error("InvalidSignature: Invalid signature")]
    InvalidSignature,
    #[error("InvalidSignatureRecoveryId: Invalid signature recovery ID")]
    InvalidSignatureRecoveryId,

    // DR contract errors
    #[error("InsufficientFunds: Insufficient funds. Required: {0}, available: {1}")]
    InsufficientFunds(Uint128, Uint128),
    #[error("DataRequestDoesNotExist {0}: Data request does not exist")]
    DataRequestDoesNotExist(String),
    #[error("DataRequestAlreadyExists: Data request already exists")]
    DataRequestAlreadyExists,
    #[error("Invalid payback address")]
    InvalidPaybackAddr,
    #[error("IneligibleExecutor: Caller is not an eligible data request executor")]
    IneligibleExecutor,
    #[error("AlreadyCommitted: Caller has already committed on this data request")]
    AlreadyCommitted,
    #[error("RevealNotStarted: Reveal stage has not started yet")]
    RevealNotStarted,
    #[error("RevealStarted: Cannot commit after reveal stage has started")]
    RevealStarted,
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

    #[error("FromHex: Invalid hexadecimal input: {0}")]
    FromHex(#[from] FromHexError),

    #[error(transparent)]
    Common(#[from] seda_common::error::Error),

    #[error("Invalid hash length `{0}` expected 32 bytes")]
    InvalidHashLength(usize),
    #[error("Invalid public key length `{0}` expected 33 bytes")]
    InvalidPublicKeyLength(usize),
}

#[cfg(test)]
impl From<StdError> for ContractError {
    fn from(err: StdError) -> Self {
        ContractError::Std(err.to_string())
    }
}
