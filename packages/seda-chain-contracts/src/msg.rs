use std::collections::HashMap;

use crate::state::{DataRequest, DataRequestExecutor, DataResult, Reveal};
use crate::types::{Bytes, Commitment, Hash, Memo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest {
        posted_dr: PostDataRequestArgs,
    },
    CommitDataResult {
        dr_id: Hash,
        commitment: String,
        sender: Option<String>,
    },
    RevealDataResult {
        dr_id: Hash,
        reveal: Reveal,
        sender: Option<String>,
    },
    RegisterDataRequestExecutor {
        p2p_multi_address: Option<String>,
        sender: Option<String>,
    },
    UnregisterDataRequestExecutor {
        sender: Option<String>,
    },
    DepositAndStake {
        sender: Option<String>,
    },
    Unstake {
        amount: u128,
        sender: Option<String>,
    },
    Withdraw {
        amount: u128,
        sender: Option<String>,
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
    #[returns(GetCommittedDataResultResponse)]
    GetCommittedDataResult { dr_id: Hash, executor: Addr },
    #[returns(GetCommittedDataResultsResponse)]
    GetCommittedDataResults { dr_id: Hash },
    #[returns(GetRevealedDataResultResponse)]
    GetRevealedDataResult { dr_id: Hash, executor: Addr },
    #[returns(GetRevealedDataResultsResponse)]
    GetRevealedDataResults { dr_id: Hash },
    #[returns(GetResolvedDataResultResponse)]
    GetResolvedDataResult { dr_id: Hash },
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
pub struct GetCommittedDataResultsResponse {
    pub value: HashMap<String, Commitment>,
}

#[cw_serde]
pub struct GetRevealedDataResultResponse {
    pub value: Option<Reveal>,
}

#[cw_serde]
pub struct GetRevealedDataResultsResponse {
    pub value: HashMap<String, Reveal>,
}

#[cw_serde]
pub struct GetResolvedDataResultResponse {
    pub value: DataResult,
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
    pub value: Vec<String>,
}

#[cw_serde]
pub struct PostDataRequestArgs {
    pub dr_id: Hash,
    pub dr_binary_id: Hash,
    pub tally_binary_id: Hash,
    pub dr_inputs: Bytes,
    pub tally_inputs: Bytes,
    pub memo: Memo,
    pub replication_factor: u16,
    pub gas_price: u128,
    pub gas_limit: u128,
    pub seda_payload: Bytes,
    pub payback_address: Bytes,
}
