use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    NewEntry {
        key: String,
        binary: cosmwasm_std::Binary,
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