#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
#[allow(unused_imports)]
use common::msg::{
    GetCommittedDataResultResponse, GetCommittedDataResultsResponse, GetContractResponse,
    GetDataRequestExecutorResponse, GetDataRequestResponse, GetDataRequestsFromPoolResponse,
    GetResolvedDataResultResponse, GetRevealedDataResultResponse, GetRevealedDataResultsResponse,
    GetStakingConfigResponse, IsDataRequestExecutorEligibleResponse, PostDataRequestArgs,
};
use common::state::RevealBody;
use common::types::{Bytes, Hash, Secpk256k1PublicKey, Signature};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
}

#[cw_serde]
pub enum ProxyExecuteMsg {
    // Owner
    // These can only be called if these are not already set. Otherwise, a sudo message must be used.
    SetDataRequests {
        contract: String,
    },
    SetStaking {
        contract: String,
    },

    // Delegated calls to contracts

    // DataRequests
    PostDataRequest {
        posted_dr: PostDataRequestArgs,
        seda_payload: Bytes,
        payback_address: Bytes,
    },
    CommitDataResult {
        dr_id: Hash,
        commitment: Hash,
        signature: Signature,
    },
    RevealDataResult {
        dr_id: Hash,
        reveal: RevealBody,
        signature: Signature,
    },
    // Staking
    RegisterDataRequestExecutor {
        signature: Signature,
        memo: Option<String>,
    },
    UnregisterDataRequestExecutor {
        signature: Signature,
    },
    DepositAndStake {
        signature: Signature,
    },
    Unstake {
        signature: Signature,
        amount: u128,
    },
    Withdraw {
        signature: Signature,
        amount: u128,
    },
    AddToAllowlist {
        address: Addr,
    },
    RemoveFromAllowlist {
        address: Addr,
    },
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
        limit: Option<u128>,
    },
    #[returns(GetCommittedDataResultResponse)]
    GetCommittedDataResult {
        dr_id: Hash,
        executor: Secpk256k1PublicKey,
    },
    #[returns(GetCommittedDataResultsResponse)]
    GetCommittedDataResults { dr_id: Hash },
    #[returns(GetRevealedDataResultResponse)]
    GetRevealedDataResult {
        dr_id: Hash,
        executor: Secpk256k1PublicKey,
    },
    #[returns(GetRevealedDataResultsResponse)]
    GetRevealedDataResults { dr_id: Hash },
    #[returns(GetResolvedDataResultResponse)]
    GetResolvedDataResult { dr_id: Hash },

    // Staking
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Secpk256k1PublicKey },
    #[returns(IsDataRequestExecutorEligibleResponse)]
    IsDataRequestExecutorEligible { executor: Secpk256k1PublicKey },
    #[returns(GetStakingConfigResponse)]
    GetStakingConfig,
}

#[cw_serde]
pub enum ProxySudoMsg {
    SetDataRequests { contract: String },
    SetStaking { contract: String },
}
