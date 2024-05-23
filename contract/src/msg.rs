use cosmwasm_schema::cw_serde;
use semver::Version;

use crate::types::{Bytes, Hash, Memo};

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
