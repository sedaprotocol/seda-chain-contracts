use std::collections::{HashMap, HashSet};

use cw_storage_plus::{Key, PrimaryKey};
use semver::Version;
use sha3::{Digest, Keccak256};

use super::*;

pub mod execute;
pub mod query;
pub mod state;

#[cfg(test)]
pub mod test_helpers;
#[cfg(test)]
mod tests;

#[cw_serde]
pub enum DataRequestStatus {
    Committing,
    Revealing,
    Tallying,
    Resolved,
}

impl DataRequestStatus {
    pub fn is_resolved(&self) -> bool {
        matches!(self, DataRequestStatus::Resolved)
    }
}

impl<'a> PrimaryKey<'a> for &'a DataRequestStatus {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = &'static str;
    type SuperSuffix = &'static str;

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(
            match self {
                DataRequestStatus::Committing => "committing",
                DataRequestStatus::Revealing => "revealing",
                DataRequestStatus::Tallying => "tallying",
                DataRequestStatus::Resolved => "resolved",
            }
            .as_bytes(),
        )]
    }
}

/// Represents a data request at creation time
#[cw_serde]
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
    pub reveals:         HashMap<String, RevealBody>,
}

impl DataRequest {
    pub fn has_committer(&self, public_key: &str) -> bool {
        self.commits.contains_key(public_key)
    }

    pub fn get_commitment(&self, public_key: &str) -> Option<&Hash> {
        self.commits.get(public_key)
    }

    pub fn has_revealer(&self, public_key: &str) -> bool {
        self.reveals.contains_key(public_key)
    }

    pub fn reveal_started(&self) -> bool {
        self.commits.len() >= self.replication_factor as usize
    }

    pub fn reveal_over(&self) -> bool {
        self.reveals.len() >= self.replication_factor as usize
    }

    pub fn get_reveal(&self, public_key: &str) -> Option<&RevealBody> {
        self.reveals.get(public_key)
    }
}

#[cw_serde]
pub enum DR {
    Request(Box<DataRequest>),
    Result(DataResult),
}

impl From<DataRequest> for DR {
    fn from(dr: DataRequest) -> Self {
        DR::Request(Box::new(dr))
    }
}

impl From<DataResult> for DR {
    fn from(dr: DataResult) -> Self {
        DR::Result(dr)
    }
}

/// Represents a resolved data result
#[cw_serde]
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
    pub gas_used:     u128,
    /// Result from Tally WASM binary execution
    pub result:       Vec<u8>,

    // Fields from Data Request Execution
    /// Payback address set by the relayer
    pub payback_address: Vec<u8>,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload:    Vec<u8>,
}

impl Hasher for DataResult {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.version.hash());
        hasher.update(self.dr_id);
        hasher.update(self.block_height.to_be_bytes());
        hasher.update(self.exit_code.to_be_bytes());
        hasher.update(self.result.hash());
        hasher.update(&self.payback_address);
        hasher.update(self.seda_payload.hash());
        hasher.finalize().into()
    }
}

/// A revealed data request result that is hashed and signed by the executor
#[cw_serde]
pub struct RevealBody {
    pub salt:      [u8; 32],
    pub exit_code: u8,
    pub gas_used:  u128,
    pub reveal:    Vec<u8>,
}

impl Hasher for RevealBody {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.salt);
        hasher.update(self.exit_code.to_be_bytes());
        hasher.update(self.gas_used.to_be_bytes());
        hasher.update(self.reveal.hash());
        hasher.finalize().into()
    }
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
