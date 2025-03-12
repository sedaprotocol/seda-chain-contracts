use crate::types::U128;

pub mod expire_data_requests;
pub mod remove_requests;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DistributionExecutorReward {
    /// The amount to burn
    pub amount:   U128,
    /// The identity to reward.
    pub identity: String,
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DistributionBurn {
    /// The amount to burn
    pub amount: U128,
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DistributionDataProxyReward {
    /// The address to send the funds to.
    pub payout_address: String,
    /// The amount to send to the address.
    pub amount:         U128,
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum DistributionMessage {
    /// For burning funds
    Burn(DistributionBurn),
    /// For rewarding an executor
    ExecutorReward(DistributionExecutorReward),
    /// For rewarding a data proxy
    DataProxyReward(DistributionDataProxyReward),
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum SudoMsg {
    RemoveDataRequests(remove_requests::Sudo),
    ExpireDataRequests(expire_data_requests::Sudo),
}

impl From<SudoMsg> for crate::msgs::SudoMsg {
    fn from(value: SudoMsg) -> Self {
        Self::DataRequest(value)
    }
}
