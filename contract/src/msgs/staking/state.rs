use seda_common::msgs::staking::{Staker, StakingConfig};

use crate::enumerable_status_map;

use super::*;

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// A map of stakers (of address to info).
pub const STAKERS: Map<&PublicKey, Staker> = Map::new("data_request_executors");
