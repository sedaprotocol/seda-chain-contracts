use crate::state::{DataRequest, DataRequestExecutor, Reveal};
use crate::types::{Commitment, Hash, Input, Memo, PayloadItem};
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
    PostDataRequest {
        dr_id: Hash,

        dr_binary_id: Hash,
        tally_binary_id: Hash,
        dr_inputs: Vec<Input>,
        tally_inputs: Vec<Input>,
        memo: Memo,
        replication_factor: u16,

        gas_price: u128,
        gas_limit: u128,

        payload: Vec<PayloadItem>,
    },
    CommitDataResult {
        dr_id: Hash,
        commitment: String,
    },
    RevealDataResult {
        dr_id: Hash,
        reveal: Reveal,
    },
    RegisterDataRequestExecutor {
        p2p_multi_address: Option<String>,
    },
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
    #[returns(GetCommittedDataResultResponse)]
    GetCommittedDataResult { dr_id: Hash, executor: Addr },
    #[returns(GetRevealedDataResultResponse)]
    GetRevealedDataResult { dr_id: Hash, executor: Addr },
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
pub struct GetCommittedDataResultResponse {
    pub value: Option<Commitment>,
}

#[cw_serde]
pub struct GetRevealedDataResultResponse {
    pub value: Option<Reveal>,
}
#[cw_serde]
pub struct GetIdsResponse {
    pub value: Vec<Hash>,
}

#[cw_serde]
pub struct GetDataRequestExecutorResponse {
    pub value: Option<DataRequestExecutor>,
}

#[cw_serde]
pub struct GetCommittedExecutorsResponse {
    pub value: Vec<Addr>,
}
