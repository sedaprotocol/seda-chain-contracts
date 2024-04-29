use crate::enumerable_map;
use crate::types::EnumerableMap;
use common::state::{DataRequest, DataResult};
use common::types::Hash;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Upon posting a data request, it is added to this map with a ID
pub const DATA_REQUESTS_POOL: EnumerableMap<Hash, DataRequest> =
    enumerable_map!("data_request_pool");

/// Upon executing a data request, the result is added to this map with a unique ID
pub const DATA_RESULTS: Map<Hash, DataResult> = Map::new("data_results_pool");

/// Address of the token used for deposit for posting a data request
pub const TOKEN: Item<String> = Item::new("token");

/// Address of proxy contract which has permission to set the sender on one's behalf
pub const PROXY_CONTRACT: Item<Addr> = Item::new("proxy_contract");
