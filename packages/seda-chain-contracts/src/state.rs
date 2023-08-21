use std::collections::HashMap;

use crate::types::{Commitment, Hash, Input, Memo, PayloadItem};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a data request at creation time
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    pub dr_id: Hash,

    pub dr_binary_id: Hash,
    pub tally_binary_id: Hash,
    pub dr_inputs: Vec<Input>,
    pub tally_inputs: Vec<Input>,

    pub memo: Memo,
    pub replication_factor: u16,

    // set by dr creator
    pub gas_price: u128,
    pub gas_limit: u128,

    // set by relayer and SEDA protocol
    pub payload: Vec<PayloadItem>,

    // set by protocol
    pub commits: HashMap<Addr, Commitment>,
    pub reveals: HashMap<Addr, Reveal>,
}

/// Represents a resolved data result
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataResult {
    pub result_id: Hash,

    pub dr_id: Hash,
    pub exit_code: u8,
    pub result: Vec<u8>,
    pub block_height: u128,

    pub payload: Vec<PayloadItem>,
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
