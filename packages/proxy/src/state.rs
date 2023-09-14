use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// Token denomination used for data request executor staking and deposit for posting a data request
pub const TOKEN: Item<String> = Item::new("token");

/// Contract address of `data-requests` contract
pub const DATA_REQUESTS: Item<Addr> = Item::new("data_requests");

/// Contract address of `staking` contract
pub const STAKING: Item<Addr> = Item::new("staking");
