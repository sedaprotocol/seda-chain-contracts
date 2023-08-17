use crate::state::{DataRequest, DataRequestExecutor, DataResult};
use crate::types::Hash;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub wasm_storage_contract_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest {
        dr_id: Hash,
        value: String,
        nonce: u128,
        chain_id: u128,
        wasm_id: Vec<u8>,
        wasm_args: Vec<Vec<u8>>,
    },
    PostDataResult {
        dr_id: Hash,
        result: String,
    },
    RegisterDataRequestExecutor {
        p2p_multi_address: Option<String>,
    },
    UnregisterDataRequestExecutor {},
    DepositAndStake,
    Unstake {
        amount: u128,
    },
    Withdraw {
        amount: u128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetDataRequestResponse)]
    GetDataRequest { dr_id: Hash },
    #[returns(GetDataRequestsFromPoolResponse)]
    GetDataRequestsFromPool {
        position: Option<u128>,
        limit: Option<u32>,
    },
    #[returns(GetDataResultResponse)]
    GetDataResult { dr_id: Hash },
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Addr },
}

#[cw_serde]
pub struct GetDataRequestResponse {
    pub value: Option<DataRequest>,
}

#[cw_serde]
pub struct GetDataRequestsFromPoolResponse {
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
