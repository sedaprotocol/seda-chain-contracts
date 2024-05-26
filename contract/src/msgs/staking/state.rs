use cw_storage_plus::{Item, Map};

use super::{Staker, StakingConfig};
use crate::types::PublicKey;

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// A map of stakers (of address to info).
pub const STAKERS: Map<&PublicKey, Staker> = Map::new("data_request_executors");
