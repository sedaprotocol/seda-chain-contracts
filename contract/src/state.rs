use cw_storage_plus::{Item, Map};

use crate::{msgs::data_requests::DataRequest, types::Hash};

/// Token denom used for staking (e.g., `aseda`).
pub const TOKEN: Item<String> = Item::new("token");

// region: data requests
pub const DATA_REQUESTS: Map<&Hash, DataRequest> = Map::new("data_results_pool");
// endregion: data requests
