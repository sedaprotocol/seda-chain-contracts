use crate::state::{DataRequest, DataRequestExecutor, DataResult, Reveal, StakingConfig};
use crate::types::{Bytes, Commitment, Hash, Memo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use semver::Version;
use std::collections::HashMap;

#[cw_serde]
pub struct PostDataRequestArgs {
    pub version: Version,
    pub dr_id: Hash,
    pub dr_binary_id: Hash,
    pub tally_binary_id: Hash,
    pub dr_inputs: Bytes,
    pub tally_inputs: Bytes,
    pub memo: Memo,
    pub replication_factor: u16,
    pub gas_price: u128,
    pub gas_limit: u128,
    pub tally_gas_limit: u128,
    pub seda_payload: Bytes,
    pub payback_address: Bytes,
}

#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum DataRequestsExecuteMsg {
    PostDataRequest {
        posted_dr: PostDataRequestArgs,
    },
    CommitDataResult {
        dr_id: Hash,
        commitment: Hash,
        sender: Option<String>,
    },
    RevealDataResult {
        dr_id: Hash,
        reveal: Reveal,
        sender: Option<String>,
    },
}

#[cw_serde]
pub enum StakingExecuteMsg {
    RegisterDataRequestExecutor {
        memo: Option<String>,
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
    GetCommittedDataResult { dr_id: Hash, executor: Addr },
    #[returns(GetCommittedDataResultsResponse)]
    GetCommittedDataResults { dr_id: Hash },
    #[returns(GetRevealedDataResultResponse)]
    GetRevealedDataResult { dr_id: Hash, executor: Addr },
    #[returns(GetRevealedDataResultsResponse)]
    GetRevealedDataResults { dr_id: Hash },
    #[returns(GetResolvedDataResultResponse)]
    GetResolvedDataResult { dr_id: Hash },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum StakingQueryMsg {
    #[returns(GetDataRequestExecutorResponse)]
    GetDataRequestExecutor { executor: Addr },
    #[returns(IsDataRequestExecutorEligibleResponse)]
    IsDataRequestExecutorEligible { executor: Addr },
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
pub struct GetDataRequestExecutorResponse {
    pub value: Option<DataRequestExecutor>,
}

#[cw_serde]
pub struct GetCommittedExecutorsResponse {
    pub value: Vec<String>,
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
