use crate::types::{Bytes, U128};

/// A data request executor with staking info and optional p2p multi address
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(
    not(feature = "cosmwasm"),
    derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)
)]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Staker {
    pub memo:                      Option<Bytes>,
    pub tokens_staked:             U128,
    pub tokens_pending_withdrawal: U128,
}

/// Governance-controlled staking configuration parameters
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(
    not(feature = "cosmwasm"),
    derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)
)]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register:               U128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: U128,
    /// Whether the allowlist is enabled
    pub allowlist_enabled:                       bool,
}

impl From<StakingConfig> for crate::msgs::ExecuteMsg {
    fn from(config: StakingConfig) -> Self {
        super::execute::ExecuteMsg::SetStakingConfig(config).into()
    }
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(
    not(feature = "cosmwasm"),
    derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)
)]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct StakerAndSeq {
    pub staker: Option<Staker>,
    pub seq:    U128,
}
