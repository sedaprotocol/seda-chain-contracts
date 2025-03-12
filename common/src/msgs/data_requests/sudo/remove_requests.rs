use std::collections::HashMap;

use super::*;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Sudo {
    pub requests: HashMap<String, Vec<DistributionMessage>>,
}

impl From<Sudo> for crate::msgs::SudoMsg {
    fn from(value: Sudo) -> Self {
        super::SudoMsg::RemoveDataRequests(value).into()
    }
}
