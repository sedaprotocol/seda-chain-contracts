use common::types::Hash;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub proxy: String,
}

#[cw_serde]
pub struct PostDataRequestResponse {
    pub dr_id: Hash,
}
