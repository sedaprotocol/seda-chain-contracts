pub mod stakers_map;

use seda_common::msgs::staking::{Staker, StakingConfig};
use stakers_map::{new_stakers_map, StakersMap};

use super::*;

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// A map of stakers (of address to info).
pub const STAKERS: StakersMap = new_stakers_map!("data_request_executors");
