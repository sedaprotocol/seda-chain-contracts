use cosmwasm_schema::cw_serde;

use super::PostDataRequestArgs;
use crate::types::Bytes;

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest {
        posted_dr:       PostDataRequestArgs,
        seda_payload:    Bytes,
        payback_address: Bytes,
    },
    // CommitDataResult {
    //     dr_id:      Hash,
    //     commitment: Hash,
    //     sender:     Option<String>,
    //     public_key: PublicKey,
    // },
    // RevealDataResult {
    //     dr_id:      Hash,
    //     reveal:     RevealBody,
    //     public_key: PublicKey,
    //     sender:     Option<String>,
    // },
}
