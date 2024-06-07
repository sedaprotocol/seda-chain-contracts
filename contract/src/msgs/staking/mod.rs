use super::*;

pub mod execute;
pub mod query;
pub use seda_contract_common::msgs::staking::query::QueryMsg;

pub mod state;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod test_helpers;

/// Governance-controlled configuration parameters
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register:               Uint128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: Uint128,
    /// Whether the allowlist is enabled
    pub allowlist_enabled:                       bool,
}
