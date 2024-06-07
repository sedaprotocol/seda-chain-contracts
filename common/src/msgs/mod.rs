#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "cosmwasm"))]
use serde::{Deserialize, Serialize};

use crate::types::*;

pub mod data_requests;
pub mod owner;
pub mod staking;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(Box<data_requests::execute::ExecuteMsg>),
    Staking(staking::execute::ExecuteMsg),
    Owner(owner::execute::ExecuteMsg),
}

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(QueryResponses))]
#[cfg_attr(feature = "cosmwasm", query_responses(nested))]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[serde(untagged)]
pub enum QueryMsg {
    DataRequest(data_requests::query::QueryMsg),
    Staking(staking::query::QueryMsg),
    Owner(owner::query::QueryMsg),
}

#[cfg_attr(feature = "cosmwasm", cw_serde)]
pub struct InstantiateMsg {
    pub token:    String,
    pub owner:    String,
    pub chain_id: String,
}
