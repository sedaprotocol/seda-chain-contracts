use crate::state::{DataRequest, DataResult};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest { value: String },
    PostDataResult { dr_id: u128, result: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetDataRequestResponse)]
    GetDataRequest { dr_id: u128 },
    #[returns(GetDataResultResponse)]
    GetDataResult { dr_id: u128 },
}

#[cw_serde]
pub struct GetDataRequestResponse {
    pub value: Option<DataRequest>,
}

#[cw_serde]
pub struct GetDataResultResponse {
    pub value: Option<DataResult>,
}
