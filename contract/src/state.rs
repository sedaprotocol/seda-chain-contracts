use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{
    msgs::staking::{Staker, StakingConfig},
    types::PublicKey,
};

/// Token denom used for staking (e.g., `aseda`).
pub const TOKEN: Item<String> = Item::new("token");

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// Address of staking contract owner.
pub const OWNER: Item<Addr> = Item::new("owner");

/// Address of pending staking contract owner.
pub const PENDING_OWNER: Item<Option<Addr>> = Item::new("pending_owner");

/// Allowlist of public keys that can register as a staker.
pub const ALLOWLIST: Map<&PublicKey, bool> = Map::new("allowlist");

/// A map of stakers (of address to info).
pub const STAKERS: Map<&PublicKey, Staker> = Map::new("data_request_executors");
