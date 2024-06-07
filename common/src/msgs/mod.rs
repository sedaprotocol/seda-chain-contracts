#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;

pub mod staking;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(feature = "cosmwasm", serde(untagged))]
pub enum ExecuteMsg {
    // DataRequest(Box<data_requests::execute::ExecuteMsg>),
    Staking(staking::execute::ExecuteMsg),
    // Owner(owner::execute::ExecuteMsg),
}
