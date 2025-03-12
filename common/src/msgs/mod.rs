#[cfg(not(feature = "cosmwasm"))]
use serde::Serialize;

pub mod data_requests;
pub mod owner;
pub mod staking;

#[cfg(test)]
mod assert_json;
#[cfg(test)]
pub use assert_json::*;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(Box<data_requests::execute::ExecuteMsg>),
    Staking(staking::execute::ExecuteMsg),
    Owner(owner::execute::ExecuteMsg),
}

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(cosmwasm_schema::QueryResponses))]
#[cfg_attr(feature = "cosmwasm", query_responses(nested))]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[serde(untagged)]
pub enum QueryMsg {
    DataRequest(data_requests::query::QueryMsg),
    Staking(staking::query::QueryMsg),
    Owner(owner::query::QueryMsg),
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
pub struct InstantiateMsg {
    pub token:          String,
    pub owner:          String,
    pub chain_id:       String,
    pub staking_config: Option<staking::StakingConfig>,
    pub timeout_config: Option<data_requests::TimeoutConfig>,
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[serde(untagged)]
pub enum SudoMsg {
    DataRequest(data_requests::sudo::SudoMsg),
}
