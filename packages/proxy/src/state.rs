use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// The creator of the contract who can set the initial contract addresses. After that, only sudo can change them.
pub const CONTRACT_CREATOR: Item<Addr> = Item::new("contract_creator");

/// Token denomination used for data request executor staking and deposit for posting a data request
pub const TOKEN: Item<String> = Item::new("token");

/// Contract address of `data-requests` contract
pub const DATA_REQUESTS: Item<Addr> = Item::new("data_requests");

/// Contract address of `staking` contract
pub const STAKING: Item<Addr> = Item::new("staking");
