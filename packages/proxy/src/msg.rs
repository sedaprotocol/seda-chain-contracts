#[allow(unused_imports)]
use common::msg::{
    GetCommittedDataResultResponse, GetCommittedDataResultsResponse, GetContractResponse,
    GetDataRequestExecutorResponse, GetDataRequestResponse, GetDataRequestsFromPoolResponse,
    GetResolvedDataResultResponse, GetRevealedDataResultResponse, GetRevealedDataResultsResponse,
    IsDataRequestExecutorEligibleResponse, PostDataRequestArgs,
};
use common::state::Reveal;
use common::types::Hash;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
}

#[cw_serde]
pub enum ProxyExecuteMsg {
    // Admin
    // These can only be called if these are not already set. Otherwise, a sudo message must be used.
    SetDataRequests { contract: String },
    SetStaking { contract: String },

    // Delegated calls to contracts

    // DataRequests
    PostDataRequest { posted_dr: PostDataRequestArgs },
    CommitDataResult { dr_id: Hash, commitment: String },
    RevealDataResult { dr_id: Hash, reveal: Reveal },
    // Staking
    RegisterDataRequestExecutor { p2p_multi_address: Option<String> },
    UnregisterDataRequestExecutor {},
    DepositAndStake,
    Unstake { amount: u128 },
    Withdraw { amount: u128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ProxyQueryMsg {
    // Proxy
    #[returns(GetContractResponse)]
    GetDataRequestsContract,
    #[returns(GetContractResponse)]
    GetStakingContract,
    // DataRequests
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

    // Staking
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Addr },
    #[returns(IsDataRequestExecutorEligibleResponse)]
    IsDataRequestExecutorEligible { executor: Addr },
}

#[cw_serde]
pub enum ProxySudoMsg {
    SetDataRequests { contract: String },
    SetStaking { contract: String },
}
