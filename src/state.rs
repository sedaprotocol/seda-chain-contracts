use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DataRequest {
    pub value: String,
}

/// Upon posting a data request, it is added to this map with a unique auto-incrementing ID
pub const DATA_REQUESTS_POOL: Map<u128, DataRequest> = Map::new("data_requests_pool");

/// Once resolved, data requests are moved to this map and removed from the pool
pub const DATA_RESULTS: Map<u128, DataRequest> = Map::new("data_results");

/// An auto-incrementing counter for the data requests
pub const DATA_REQUESTS_COUNT: Item<u128> = Item::new("data_requests_count");
