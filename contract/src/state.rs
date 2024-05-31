use cw_storage_plus::Item;

/// Token denom used for staking (e.g., `aseda`).
pub const TOKEN: Item<String> = Item::new("token");

/// Chain ID of the network (e.g., `seda-1`).
/// Used as a "magic number"
pub const CHAIN_ID: Item<String> = Item::new("chain_id");
