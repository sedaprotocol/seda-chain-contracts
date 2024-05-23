use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::Secp256k1PublicKey;

pub mod staking;
pub use staking::{ExecuteMsg as StakingExecuteMsg, QueryMsg as StakingQueryMsg};

#[cw_serde]
pub enum ExecuteMsg {
    Staking(StakingExecuteMsg),
    TransferOwnership {
        new_owner: String,
    },
    AcceptOwnership {},
    /// Add a user to the allowlist.
    AddToAllowlist {
        /// The public key of the person to allowlist.
        pub_key: Secp256k1PublicKey,
    },
    /// Remove a user from the allowlist.
    RemoveFromAllowlist {
        /// The public key of the person remove from allowlist.
        pub_key: Secp256k1PublicKey,
    },
}

impl From<StakingExecuteMsg> for ExecuteMsg {
    fn from(value: StakingExecuteMsg) -> Self {
        Self::Staking(value)
    }
}

// #[cw_serde]
// #[derive(QueryResponses)]
// pub enum DataRequestsQueryMsg {
//     #[returns(GetDataRequestResponse)]
//     GetDataRequest { dr_id: Hash },
//     #[returns(GetDataRequestsFromPoolResponse)]
//     GetDataRequestsFromPool {
//         position: Option<u128>,
//         limit:    Option<u128>,
//     },
//     #[returns(GetCommittedDataResultResponse)]
//     GetCommittedDataResult {
//         dr_id:    Hash,
//         executor: Secp256k1PublicKey,
//     },
//     #[returns(GetCommittedDataResultsResponse)]
//     GetCommittedDataResults { dr_id: Hash },
//     #[returns(GetRevealedDataResultResponse)]
//     GetRevealedDataResult {
//         dr_id:    Hash,
//         executor: Secp256k1PublicKey,
//     },
//     #[returns(GetRevealedDataResultsResponse)]
//     GetRevealedDataResults { dr_id: Hash },
//     #[returns(GetResolvedDataResultResponse)]
//     GetResolvedDataResult { dr_id: Hash },
// }

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cw_serde]
#[serde(untagged)]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Staking(StakingQueryMsg),
    Rest(QueryMsgRest),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgRest {
    #[returns(cosmwasm_std::Addr)]
    GetOwner,
    #[returns(Option<cosmwasm_std::Addr>)]
    GetPendingOwner,
}

impl From<StakingQueryMsg> for QueryMsg {
    fn from(value: StakingQueryMsg) -> Self {
        Self::Staking(value)
    }
}

impl From<QueryMsgRest> for QueryMsg {
    fn from(value: QueryMsgRest) -> Self {
        Self::Rest(value)
    }
}

// impl QueryResponses for QueryMsg {
//     fn response_schemas_impl() -> std::collections::BTreeMap<String, schemars::schema::RootSchema> {
//         let mut schemas = std::collections::BTreeMap::new();

//         // Merge schemas from StakingQueryMsg
//         let staking_schemas = StakingQueryMsg::response_schemas_impl();
//         for (key, value) in staking_schemas {
//             schemas.insert(key, value);
//         }

//         // Add schemas for GetOwner and GetPendingOwner
//         schemas.insert("GetOwner".to_string(), RootSchema::new::<cosmwasm_std::Addr>());
//         schemas.insert("GetPendingOwner".to_string(), RootSchema::new::<Option<cosmwasm_std::Addr>>());

//         schemas
//     }
// }
