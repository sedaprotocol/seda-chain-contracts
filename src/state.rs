use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a data request at creation time
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    pub dr_id: u128,
    pub value: String,
}

/// An resolved data request with an attached result
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataResult {
    pub dr_id: u128,
    pub value: String,
    pub result: String,
}

/// A data request executor with staking info and optional p2p multi address
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequestExecutor {
    pub p2p_multi_address: Option<String>,
    pub tokens_staked: u128,
    pub tokens_pending_withdrawal: u128,
}

/// Upon posting a data request, it is added to this map with a unique auto-incrementing ID
pub const DATA_REQUESTS_POOL: Map<u128, DataRequest> = Map::new("data_requests_pool");

/// Once resolved, data requests are moved to this map and removed from the pool
pub const DATA_RESULTS: Map<u128, DataResult> = Map::new("data_results");

/// An auto-incrementing counter for the data requests
pub const DATA_REQUESTS_COUNT: Item<u128> = Item::new("data_requests_count");

/// A map of data request executors (of address to info) that have not yet been marked as active
pub const INACTIVE_DATA_REQUEST_EXECUTORS: Map<Addr, DataRequestExecutor> =
    Map::new("inactive_data_request_executors");

/// Address of the token used for data request executor staking
pub const TOKEN: Item<String> = Item::new("token");
