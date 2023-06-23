use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataResult {
    pub value: String,
    pub result: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequestExecutor {
    pub bn254_public_key: String,
    pub multi_address: String,
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
