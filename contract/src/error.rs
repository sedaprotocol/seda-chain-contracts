use std::num::NonZero;

use cosmwasm_std::{StdError, Uint128};
use hex::FromHexError;
use thiserror::Error;

use crate::msgs::data_requests::consts::{MIN_EXEC_GAS_LIMIT, MIN_GAS_PRICE, MIN_TALLY_GAS_LIMIT};

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
    #[error("InvalidAddress: Invalid address: {0}")]
    InvalidAddress(String),
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
    #[error("Could not encode protobuf message: {0}")]
    ProtoEncode(seda_proto_common::prost::EncodeError),

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
    #[error("ZeroMinimumStakeToRegister: Minimum stake to register cannot be zero")]
    ZeroMinimumStakeToRegister,

    #[error("GasPriceTooLow: Gas price {0} is lower than the minimum {MIN_GAS_PRICE}")]
    GasPriceTooLow(Uint128),
    #[error("ExecGasLimitTooLow: Exec gas limit {0} is lower than the minimum {MIN_EXEC_GAS_LIMIT}")]
    ExecGasLimitTooLow(u64),
    #[error("TallyGasLimitTooLow: Tally gas limit {0} is lower than the minimum {MIN_TALLY_GAS_LIMIT}")]
    TallyGasLimitTooLow(u64),

    #[error("SemVer: Invalid semver: {0}")]
    SemVer(String),
    #[error("No migration needed.")]
    NoMigrationNeeded,
    #[error("Cannot downgrade contract version")]
    DowngradeNotSupported,
    #[error("Cannot reveal: Reveal data is too big for the data request")]
    RevealTooBig,
    #[error("Cannot Post Data Request: invalid {0} program id length: {1}")]
    ProgramIdInvalidLength(&'static str, usize),
    #[error("Cannot Post Data Request: {0} field is too big ({1} bytes), max allowed is {2} bytes")]
    DrFieldTooBig(&'static str, usize, NonZero<u16>),
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

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
