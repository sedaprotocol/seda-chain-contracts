use std::collections::HashMap;

use semver::Version;
use sha3::{Digest, Keccak256};

use super::*;

pub mod execute;
pub mod query;
pub mod state;
pub mod utils;

#[cfg(test)]
pub mod test_helpers;
#[cfg(test)]
mod tests;

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
    pub commits:         HashMap<String, Hash>,
    /// Reveals submitted by executors
    pub reveals:         HashMap<String, Hash>,
}

// /// Represents a resolved data result
// #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
// pub struct DataResult {
//     // DR Result
//     /// Semantic Version String
//     pub version: Version,

//     /// Data Request Identifier
//     pub dr_id:        Hash,
//     /// Block Height at which data request was finalized
//     pub block_height: u64,
//     /// Exit code of Tally WASM binary execution
//     pub exit_code:    u8,
//     /// Result from Tally WASM binary execution
//     pub result:       Bytes,

//     // Fields from Data Request Execution
//     /// Payback address set by the relayer
//     pub payback_address: Bytes,
//     /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
//     pub seda_payload:    Bytes,
// }

// /// A revealed data request result that is hashed and signed by the executor
// #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
// pub struct RevealBody {
//     pub salt:      [u8; 32],
//     pub exit_code: u8,
//     pub gas_used:  u128,
//     pub reveal:    Bytes,
// }

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

impl Hasher for PostDataRequestArgs {
    fn hash(&self) -> Hash {
        // hash non-fixed-length inputs
        let mut dr_inputs_hasher = Keccak256::new();
        dr_inputs_hasher.update(&self.dr_inputs);
        let dr_inputs_hash = dr_inputs_hasher.finalize();

        let mut tally_inputs_hasher = Keccak256::new();
        tally_inputs_hasher.update(&self.tally_inputs);
        let tally_inputs_hash = tally_inputs_hasher.finalize();

        let mut memo_hasher = Keccak256::new();
        memo_hasher.update(&self.memo);
        let memo_hash = memo_hasher.finalize();

        // hash data request
        let mut dr_hasher = Keccak256::new();
        dr_hasher.update(self.version.hash());
        dr_hasher.update(self.dr_binary_id);
        dr_hasher.update(dr_inputs_hash);
        dr_hasher.update(self.tally_binary_id);
        dr_hasher.update(tally_inputs_hash);
        dr_hasher.update(self.replication_factor.to_be_bytes());
        dr_hasher.update(self.gas_price.to_be_bytes());
        dr_hasher.update(self.gas_limit.to_be_bytes());
        dr_hasher.update(memo_hash);
        dr_hasher.finalize().into()
    }
}
