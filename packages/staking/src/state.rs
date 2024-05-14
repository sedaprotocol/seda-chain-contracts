use common::{
    state::{DataRequestExecutor, StakingConfig},
    types::Secpk256k1PublicKey,
};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Address of the token used for data request executor staking
pub const TOKEN: Item<String> = Item::new("token");

/// A map of data request executors (of address to info) that have not yet been marked as active
pub const DATA_REQUEST_EXECUTORS: Map<&Secpk256k1PublicKey, DataRequestExecutor> = Map::new("data_request_executors");

/// A map of data request executors (of Secpk256k1PublicKey to boolean) that are eligible for committee inclusion
pub const ELIGIBLE_DATA_REQUEST_EXECUTORS: Map<&Secpk256k1PublicKey, bool> =
    Map::new("eligible_data_request_executors");

/// Address of proxy contract which has permission to set the sender on one's behalf
pub const PROXY_CONTRACT: Item<Addr> = Item::new("proxy_contract");

/// Address of staking contract owner
pub const OWNER: Item<Addr> = Item::new("owner");

/// Address of pending staking contract owner
pub const PENDING_OWNER: Item<Option<Addr>> = Item::new("pending_owner");

/// Governance-controlled configuration parameters
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// Allowlist of addresses that can register as a data request executor
pub const ALLOWLIST: Map<&Secpk256k1PublicKey, bool> = Map::new("allowlist");
