use cosmwasm_std::{StdError, Uint128};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
    #[error("NotOwner: Only owner can transfer ownership")]
    NotOwner,
    #[error("NotPendingOwner: Only pending owner can accept ownership")]
    NotPendingOwner,
    #[error("NoPendingOwnerFound: No pending owner found")]
    NoPendingOwnerFound,
    #[error("NotOnAllowlist: Address is not on the allowlist")]
    NotOnAllowlist,

    // DR contract errors
    #[error("InsufficientFunds: Insufficient funds. Required: {0}, available: {1}")]
    InsufficientFunds(Uint128, Uint128),
    #[error("DataRequestAlreadyExists: Data request already exists")]
    DataRequestAlreadyExists,
    #[error("DataRequestReplicationFactorZero: Data request replication factor cannot be zero")]
    DataRequestReplicationFactorZero,
    #[error(
        "ReplicationFactorExceedsExecutorCount: The specified replication factor exceeds the available number of executors ({0})"
    )]
    DataRequestReplicationFactorTooHigh(u32),
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
    #[error("NotEnoughReveals: Not enough reveals to post the data result")]
    NotEnoughReveals,
    #[error("DataRequestExpired: Data request expired at block height {0} during {1} stage")]
    DataRequestExpired(u64, &'static str),

    #[error("FromHex: Invalid hexadecimal input: {0}")]
    FromHex(#[from] FromHexError),

    #[error(transparent)]
    Payment(#[from] cw_utils::PaymentError),

    #[error(transparent)]
    Common(#[from] seda_common::error::Error),

    #[error(transparent)]
    Overflow(#[from] cosmwasm_std::OverflowError),

    #[error("Invalid hash length `{0}` expected 32 bytes")]
    InvalidHashLength(usize),
    #[error("Invalid public key length `{0}` expected 33 bytes")]
    InvalidPublicKeyLength(usize),
    #[error("Contract paused: cannot perform operation `{0}`")]
    ContractPaused(String),
    #[error("Contract not paused: cannot unpause")]
    ContractNotPaused,
}

#[cfg(test)]
impl From<StdError> for ContractError {
    fn from(err: StdError) -> Self {
        ContractError::Std(err.to_string())
    }
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
