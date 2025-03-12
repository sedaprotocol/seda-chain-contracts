use crate::{msgs::data_requests::PostDataRequestArgs, types::Bytes};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Execute {
    pub posted_dr:       PostDataRequestArgs,
    pub seda_payload:    Bytes,
    pub payback_address: Bytes,
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::PostDataRequest(value).into()
    }
}
