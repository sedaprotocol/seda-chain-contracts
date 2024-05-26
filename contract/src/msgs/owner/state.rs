use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::types::PublicKey;

/// Address of staking contract owner.
pub const OWNER: Item<Addr> = Item::new("owner");

/// Address of pending staking contract owner.
pub const PENDING_OWNER: Item<Option<Addr>> = Item::new("pending_owner");

/// Allowlist of public keys that can register as a staker.
pub const ALLOWLIST: Map<&PublicKey, bool> = Map::new("allowlist");
