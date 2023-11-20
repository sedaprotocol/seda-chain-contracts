use crate::types::{Bytes, Commitment, Hash};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a data request at creation time
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    /// Identifier
    pub dr_id: Hash,

    // DR definition
    /// Semantic Version String
    // pub version: Version,
    /// Identifier of DR WASM binary
    pub dr_binary_id: Hash,
    /// Inputs for DR WASM binary
    pub dr_inputs: Bytes,
    /// Identifier of Tally WASM binary
    pub tally_binary_id: Hash,
    /// Inputs for Tally WASM binary
    pub tally_inputs: Bytes,
    /// Amount of required DR executors
    pub replication_factor: u16,
    /// Amount of SEDA tokens per gas unit
    pub gas_price: u128,
    /// Maximum of gas units to be used
    pub gas_limit: u128,
    /// Public info attached to DR
    pub memo: Bytes,

    // Execution Information
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload: Bytes,
    /// Commitments submitted by executors
    pub commits: HashMap<String, Commitment>,
    /// Reveals submitted by executors
    pub reveals: HashMap<String, Reveal>,
    /// hash(random seed fetched from chain, dr_id)
    pub seed_hash: Hash,
}

/// Represents a resolved data result
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataResult {
    /// Identifier
    pub result_id: Hash,

    // DR Result
    /// Semantic Version String
    // pub version: Version,

    /// Data Request Identifier
    pub dr_id: Hash,
    /// Block Height at which data request was finalized
    pub block_height: u64,
    /// Exit code of Tally WASM binary execution
    pub exit_code: u8,
    /// Result from Tally WASM binary execution
    pub result: Bytes,

    // Fields from Data Request Execution
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload: Bytes,
}

/// A revealed data request with an attached reveal and salt
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct Reveal {
    pub reveal: String,
    pub salt: String,
}

/// A data request executor with staking info and optional p2p multi address
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequestExecutor {
    pub p2p_multi_address: Option<String>,
    pub tokens_staked: u128,
    pub tokens_pending_withdrawal: u128,
}

/// Governance-controlled configuration parameters
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register: u128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: u128,
}
