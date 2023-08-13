use crate::state::{DataRequest, DataRequestExecutor, CommittedDataResult};
use crate::types::Hash;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct PostDataRequestArgs {
    pub dr_id: Hash,
    pub value: String,
    pub nonce: u128,
    pub chain_id: u128,
    pub wasm_id: Vec<u8>,
    pub wasm_args: Vec<Vec<u8>>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest { args: PostDataRequestArgs },
    PostDataResult { dr_id: Hash, result: String },
    RegisterDataRequestExecutor { p2p_multi_address: Option<String> },
    UnregisterDataRequestExecutor {},
    DepositAndStake,
    Unstake { amount: u128 },
    Withdraw { amount: u128 },
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
    pub value: Option<Vec<CommittedDataResult>>,
}

#[cw_serde]
pub struct GetDataRequestExecutorResponse {
    pub value: Option<DataRequestExecutor>,
}
