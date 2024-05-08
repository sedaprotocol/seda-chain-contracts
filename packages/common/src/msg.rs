use crate::state::{DataRequest, DataRequestExecutor, DataResult, RevealBody, StakingConfig};
use crate::types::{Bytes, Commitment, Hash, Memo, Secpk256k1PublicKey, Signature};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use semver::Version;
use std::collections::HashMap;

#[cw_serde]
pub struct PostDataRequestArgs {
    pub version: Version,
    pub dr_binary_id: Hash,
    pub dr_inputs: Bytes,
    pub tally_binary_id: Hash,
    pub tally_inputs: Bytes,
    pub replication_factor: u16,
    pub gas_price: u128,
    pub gas_limit: u128,
    pub memo: Memo,
}

#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum DataRequestsExecuteMsg {
    PostDataRequest {
        posted_dr: PostDataRequestArgs,
        seda_payload: Bytes,
        payback_address: Bytes,
    },
    CommitDataResult {
        dr_id: Hash,
        commitment: Hash,
        public_key: Secpk256k1PublicKey,
        sender: Option<String>,
    },
    RevealDataResult {
        dr_id: Hash,
        reveal: RevealBody,
        signature: Signature,
        sender: Option<String>,
    },
}

#[cw_serde]
pub enum StakingExecuteMsg {
    RegisterDataRequestExecutor {
        signature: Signature,
        memo: Option<String>,
        sender: Option<String>,
    },
    UnregisterDataRequestExecutor {
        signature: Signature,
        sender: Option<String>,
    },
    DepositAndStake {
        signature: Signature,
        sender: Option<String>,
    },
    Unstake {
        signature: Signature,
        amount: u128,
        sender: Option<String>,
    },
    Withdraw {
        signature: Signature,
        amount: u128,
        sender: Option<String>,
    },
    TransferOwnership {
        new_owner: String,
    },
    AcceptOwnership {},
    SetStakingConfig {
        config: StakingConfig,
    },
    AddToAllowlist {
        sender: Option<String>,
        address: Addr,
    },
    RemoveFromAllowlist {
        sender: Option<String>,
        address: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum DataRequestsQueryMsg {
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
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum StakingQueryMsg {
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Secpk256k1PublicKey },
    #[returns(IsDataRequestExecutorEligibleResponse)]
    IsDataRequestExecutorEligible { executor: Secpk256k1PublicKey },
    #[returns(GetStakingConfigResponse)]
    GetStakingConfig,
    #[returns(GetOwnerResponse)]
    GetOwner,
    #[returns(GetPendingOwnerResponse)]
    GetPendingOwner,
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
    pub value: HashMap<String, Commitment>, // key is hex::encode(public_key)
}

#[cw_serde]
pub struct GetRevealedDataResultResponse {
    pub value: Option<RevealBody>,
}

#[cw_serde]
pub struct GetRevealedDataResultsResponse {
    pub value: HashMap<String, RevealBody>, // key is hex::encode(public_key)
}

#[cw_serde]
pub struct GetResolvedDataResultResponse {
    pub value: DataResult,
}

#[cw_serde]
pub struct GetDataRequestExecutorResponse {
    pub value: Option<DataRequestExecutor>,
}

#[cw_serde]
pub struct GetCommittedExecutorsResponse {
    pub value: Vec<Secpk256k1PublicKey>,
}

#[cw_serde]
pub struct IsDataRequestExecutorEligibleResponse {
    pub value: bool,
}

#[cw_serde]
pub struct GetContractResponse {
    pub value: String,
}

#[cw_serde]
pub struct GetStakingConfigResponse {
    pub value: StakingConfig,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub proxy: String,
    pub owner: String,
}

#[cw_serde]
pub struct GetOwnerResponse {
    pub value: Addr,
}

#[cw_serde]
pub struct GetPendingOwnerResponse {
    pub value: Option<Addr>,
}
