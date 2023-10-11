use common::state::{Config, DataRequestExecutor};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Address of the token used for data request executor staking
pub const TOKEN: Item<String> = Item::new("token");

/// A map of data request executors (of address to info) that have not yet been marked as active
pub const DATA_REQUEST_EXECUTORS: Map<Addr, DataRequestExecutor> =
    Map::new("data_request_executors");

/// A map of data request executors (of address to info) that are eligible for committee inclusion
pub const ELIGIBLE_DATA_REQUEST_EXECUTORS: Map<Addr, bool> =
    Map::new("eligible_data_request_executors");

/// Address of proxy contract which has permission to set the sender on one's behalf
pub const PROXY_CONTRACT: Item<Addr> = Item::new("proxy_contract");

/// Governance-controlled configuration parameters
pub const CONFIG: Item<Config> = Item::new("config");
