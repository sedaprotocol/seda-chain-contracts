use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use semver::Version;

use crate::{
    state::{DataRequest, DataResult, RevealBody},
    types::{Bytes, Commitment, Hash, Memo, Secp256k1PublicKey},
};

#[cw_serde]
pub struct PostDataRequestArgs {
    pub version:            Version,
    pub dr_binary_id:       Hash,
    pub dr_inputs:          Bytes,
    pub tally_binary_id:    Hash,
    pub tally_inputs:       Bytes,
    pub replication_factor: u16,
    pub gas_price:          u128,
    pub gas_limit:          u128,
    pub memo:               Memo,
}

// #[allow(clippy::large_enum_variant)]
// #[cw_serde]
// pub enum DataRequestsExecuteMsg {
//     PostDataRequest {
//         posted_dr:       PostDataRequestArgs,
//         seda_payload:    Bytes,
//         payback_address: Bytes,
//     },
//     CommitDataResult {
//         dr_id:      Hash,
//         commitment: Hash,
//         sender:     Option<String>,
//         signature:  Signature,
//     },
//     RevealDataResult {
//         dr_id:     Hash,
//         reveal:    RevealBody,
//         signature: Signature,
//         sender:    Option<String>,
//     },
// }

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
pub struct GetCommittedExecutorsResponse {
    pub value: Vec<Secp256k1PublicKey>,
}

#[cw_serde]
pub struct GetContractResponse {
    pub value: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    // pub proxy: String,
    pub owner: String,
}
