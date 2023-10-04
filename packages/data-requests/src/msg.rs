use common::types::Hash;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct PostDataRequestResponse {
    pub dr_id: Hash,
}
