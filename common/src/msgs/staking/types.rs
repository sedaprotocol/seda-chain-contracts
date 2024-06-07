#[cfg(feature = "cosmwasm")]
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "cosmwasm")]
type U128 = Uint128;
#[cfg(not(feature = "cosmwasm"))]
type U128 = String;

/// A data request executor with staking info and optional p2p multi address
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct Staker {
    pub memo:                      Option<String>,
    pub tokens_staked:             U128,
    pub tokens_pending_withdrawal: U128,
}

/// Governance-controlled configuration parameters
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register:               U128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: U128,
    /// Whether the allowlist is enabled
    pub allowlist_enabled:                       bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakerAndSeq {
    pub staker: Option<Staker>,
    pub seq:    U128,
}
