use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::*;

pub mod execute;
pub mod query;
pub mod state;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod test_helpers;

/// A data request executor with staking info and optional p2p multi address
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct Staker {
    pub memo:                      Option<String>,
    pub tokens_staked:             u128,
    pub tokens_pending_withdrawal: u128,
}

/// Governance-controlled configuration parameters
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register:               u128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: u128,
    /// Whether the allowlist is enabled
    pub allowlist_enabled:                       bool,
}
