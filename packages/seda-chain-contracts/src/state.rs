use crate::types::Hash;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a data request at creation time
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    pub dr_id: Hash,
    pub nonce: u128,
    pub value: String,
    pub chain_id: u128,
    pub wasm_id: Vec<u8>,
    pub wasm_args: Vec<Vec<u8>>,
}

/// A committed data request with an attached result
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct CommittedDataResult {
    pub dr_id: Hash,
    pub nonce: u128,
    pub value: String,
    pub chain_id: u128,
    pub executor: Addr,
    pub commitment: Hash,
}

/// A revealed data request with an attached reveal and salt
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct RevealedDataResult {
    pub dr_id: Hash,
    pub nonce: u128,
    pub value: String,
    pub chain_id: u128,
    pub executor: Addr,
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
pub const DATA_REQUESTS_POOL: Map<Hash, DataRequest> = Map::new("data_requests_pool");

/// A map of data requests in the pool by nonce
pub const DATA_REQUESTS_BY_NONCE: Map<u128, Hash> = Map::new("DATA_REQUESTS_BY_NONCE");

/// Once committed, data requests are moved to this map and removed from the DATA_REQUESTS_POOL
pub const COMMITTED_DATA_RESULTS: Map<Hash, Vec<CommittedDataResult>> =
    Map::new("committed_data_results");

/// Once revealed, committed data requests are moved to this map and removed from the COMMITTED_DATA_RESULTS
pub const REVEALED_DATA_RESULTS: Map<Hash, Vec<RevealedDataResult>> =
    Map::new("revealed_data_results");

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
