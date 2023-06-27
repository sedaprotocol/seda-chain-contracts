use crate::state::{DataRequest, DataRequestExecutor, DataResult};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest {
        value: String,
    },
    PostDataResult {
        dr_id: u128,
        result: String,
    },
    RegisterDataRequestExecutor {
        bn254_public_key: String,
        multi_address: String,
    },
    UnregisterDataRequestExecutor {},
    Stake,
    Unstake {
        amount: u128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetDataRequestResponse)]
    GetDataRequest { dr_id: u128 },
    #[returns(GetDataRequestsResponse)]
    GetDataRequests {
        position: Option<u128>,
        limit: Option<u32>,
    },
    #[returns(GetDataResultResponse)]
    GetDataResult { dr_id: u128 },
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Addr },
}

#[cw_serde]
pub struct GetDataRequestResponse {
    pub value: Option<DataRequest>,
}

#[cw_serde]
pub struct GetDataRequestsResponse {
    pub value: Vec<DataRequest>,
}

#[cw_serde]
pub struct GetDataResultResponse {
    pub value: Option<DataResult>,
}

#[cw_serde]
pub struct GetDataRequestExecutorResponse {
    pub value: Option<DataRequestExecutor>,
}
