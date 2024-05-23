use std::collections::HashMap;

use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::types::{Bytes, Commitment, Hash, Memo};

/// Represents a data request at creation time
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequest {
    /// Identifier
    pub id: Hash,

    // DR definition
    /// Semantic Version String
    pub version:            Version,
    /// Identifier of DR WASM binary
    pub dr_binary_id:       Hash,
    /// Inputs for DR WASM binary
    pub dr_inputs:          Bytes,
    /// Identifier of Tally WASM binary
    pub tally_binary_id:    Hash,
    /// Inputs for Tally WASM binary
    pub tally_inputs:       Bytes,
    /// Amount of required DR executors
    pub replication_factor: u16,
    /// Amount of SEDA tokens per gas unit
    pub gas_price:          u128,
    /// Maximum of gas units to be used by data request executors to resolve a data request
    pub gas_limit:          u128,
    /// Public info attached to DR
    pub memo:               Memo,

    // Execution Information
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload:    Bytes,
    /// Commitments submitted by executors
    pub commits:         HashMap<String, Commitment>,
    /// Reveals submitted by executors
    pub reveals:         HashMap<String, RevealBody>,
}

/// Represents a resolved data result
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataResult {
    // DR Result
    /// Semantic Version String
    pub version: Version,

    /// Data Request Identifier
    pub dr_id:        Hash,
    /// Block Height at which data request was finalized
    pub block_height: u64,
    /// Exit code of Tally WASM binary execution
    pub exit_code:    u8,
    /// Result from Tally WASM binary execution
    pub result:       Bytes,

    // Fields from Data Request Execution
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload:    Bytes,
}

/// A revealed data request result that is hashed and signed by the executor
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct RevealBody {
    pub salt:      [u8; 32],
    pub exit_code: u8,
    pub gas_used:  u128,
    pub reveal:    Bytes,
}

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

#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum DataRequestsExecuteMsg {
    PostDataRequest {
        posted_dr:       PostDataRequestArgs,
        seda_payload:    Bytes,
        payback_address: Bytes,
    },
    CommitDataResult {
        dr_id:      Hash,
        commitment: Hash,
        sender:     Option<String>,
        signature:  Signature,
    },
    RevealDataResult {
        dr_id:     Hash,
        reveal:    RevealBody,
        signature: Signature,
        sender:    Option<String>,
    },
}
