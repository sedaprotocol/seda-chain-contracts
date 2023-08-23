use std::collections::HashMap;

use crate::types::{Bytes, Commitment, Hash};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
// use semver::Version;
use serde::{Deserialize, Serialize};

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

/// Upon posting a data request, it is added to this map with a unique auto-incrementing ID
pub const DATA_REQUESTS: Map<Hash, DataRequest> = Map::new("data_requests_pool");

/// Upon executing a data request, teh result is added to this map with a unique auto-incrementing ID
pub const DATA_RESULTS: Map<Hash, DataResult> = Map::new("data_results_pool");

/// A map of data requests in the pool by nonce
pub const DATA_REQUESTS_BY_NONCE: Map<u128, Hash> = Map::new("DATA_REQUESTS_BY_NONCE");

/// An auto-incrementing counter for the data requests
pub const DATA_REQUESTS_COUNT: Item<u128> = Item::new("data_requests_count");

/// A map of data request executors (of address to info) that have not yet been marked as active
pub const DATA_REQUEST_EXECUTORS: Map<Addr, DataRequestExecutor> =
    Map::new("data_request_executors");

/// Address of the token used for data request executor staking
pub const TOKEN: Item<String> = Item::new("token");

/// A map of data request executors (of address to info) that are eligible for committee inclusion
pub const ELIGIBLE_DATA_REQUEST_EXECUTORS: Map<Addr, bool> =
    Map::new("eligible_data_request_executors");
