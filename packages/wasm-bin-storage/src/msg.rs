use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    NewEntry {
        binary: Vec<u8>,
        description: String,
    },
    DeleteEntry {
        key: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::BinaryStruct)]
    QueryEntry { key: String },
}
