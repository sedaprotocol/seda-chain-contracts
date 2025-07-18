use std::{collections::HashMap, num::NonZero};

#[cfg(not(feature = "cosmwasm"))]
use base64::{prelude::BASE64_STANDARD, Engine};
use semver::Version;
#[cfg(not(feature = "cosmwasm"))]
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::{error::Result, types::*};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct PostRequestResponsePayload {
    pub dr_id:  String,
    pub height: u64,
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum DataRequestStatus {
    Committing,
    Revealing,
    Tallying,
}

/// Represents a data request at creation time
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Deserialize, Clone, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DataRequestBase {
    /// Identifier
    pub id: String,

    // DR definition
    /// Semantic Version String
    pub version:            Version,
    /// Identifier of DR WASM binary
    pub exec_program_id:    String,
    /// Inputs for DR WASM binary
    pub exec_inputs:        Bytes,
    /// The limit to how much gas should be used for executing the DR
    pub exec_gas_limit:     u64,
    /// Identifier of Tally WASM binary
    pub tally_program_id:   String,
    /// Inputs for Tally WASM binary
    pub tally_inputs:       Bytes,
    /// The limit to how much gas should be used for tallying the DR
    pub tally_gas_limit:    u64,
    /// Amount of required DR executors
    pub replication_factor: u16,
    /// Filter applied before tally execution
    pub consensus_filter:   Bytes,
    /// Amount of SEDA tokens per gas unit
    pub gas_price:          U128,
    /// Public info attached to DR
    pub memo:               Bytes,

    // Execution Information
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload:    Bytes,
    /// Commitments submitted by executors
    pub commits:         HashMap<String, Hash>,

    /// The height data request was posted. Used for commitment.
    pub height: u64,

    /// The actual gas price derived from the funds sent (funds /
    /// total_gas_limit). This may be higher than the minimum gas_price.
    pub posted_gas_price: U128,
}

impl DataRequestBase {
    pub fn has_committer(&self, public_key: &str) -> bool {
        self.commits.contains_key(public_key)
    }

    pub fn get_commitment(&self, public_key: &str) -> Option<&Hash> {
        self.commits.get(public_key)
    }

    pub fn reveal_started(&self) -> bool {
        self.commits.len() >= self.replication_factor as usize
    }
}

#[cfg(feature = "cosmwasm")]
#[cosmwasm_schema::cw_serde]
pub struct DataRequestContract {
    #[serde(flatten)]
    pub base:    DataRequestBase,
    pub reveals: std::collections::HashSet<String>,
}

#[cfg(feature = "cosmwasm")]
impl DataRequestContract {
    pub fn has_revealer(&self, public_key: &str) -> bool {
        self.reveals.contains(public_key)
    }

    pub fn is_tallying(&self) -> bool {
        self.reveals.len() >= self.base.replication_factor as usize
    }
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Deserialize, Clone, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DataRequestResponse {
    #[serde(flatten)]
    pub base:    DataRequestBase,
    /// Reveals submitted by executors
    pub reveals: HashMap<String, RevealBody>,
}

impl DataRequestResponse {
    pub fn has_revealer(&self, public_key: &str) -> bool {
        self.reveals.contains_key(public_key)
    }

    pub fn is_tallying(&self) -> bool {
        self.reveals.len() >= self.base.replication_factor as usize
    }

    pub fn get_reveal(&self, public_key: &str) -> Option<&RevealBody> {
        self.reveals.get(public_key)
    }
}

/// A revealed data request result that is hashed and signed by the executor
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Deserialize, Clone, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct RevealBody {
    pub dr_id:             String,
    pub dr_block_height:   u64,
    pub exit_code:         u8,
    pub gas_used:          u64,
    pub reveal:            Bytes,
    pub proxy_public_keys: Vec<String>,
}

impl TryHashSelf for RevealBody {
    fn try_hash(&self) -> Result<Hash> {
        let mut reveal_hasher = Keccak256::new();
        #[cfg(feature = "cosmwasm")]
        reveal_hasher.update(self.reveal.as_slice());
        #[cfg(not(feature = "cosmwasm"))]
        reveal_hasher.update(BASE64_STANDARD.decode(&self.reveal)?);
        let reveal_hash = reveal_hasher.finalize();

        let mut hasher = Keccak256::new();
        hasher.update(hex::decode(&self.dr_id)?);
        hasher.update(self.dr_block_height.to_be_bytes());
        hasher.update(self.exit_code.to_be_bytes());
        hasher.update(self.gas_used.to_be_bytes());
        hasher.update(reveal_hash);
        let proxy_public_keys: &[String] = &self.proxy_public_keys;
        hasher.update(proxy_public_keys.hash());

        Ok(hasher.finalize().into())
    }
}

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
#[cfg_attr(all(not(feature = "cosmwasm"), test), derive(Deserialize))]
pub struct PostDataRequestArgs {
    pub version:            Version,
    pub exec_program_id:    String,
    pub exec_inputs:        Bytes,
    pub exec_gas_limit:     u64,
    pub tally_program_id:   String,
    pub tally_inputs:       Bytes,
    pub tally_gas_limit:    u64,
    pub replication_factor: u16,
    pub consensus_filter:   Bytes,
    pub gas_price:          U128,
    pub memo:               Bytes,
}

impl TryHashSelf for PostDataRequestArgs {
    fn try_hash(&self) -> Result<Hash> {
        // hash non-fixed-length inputs
        let mut exec_inputs_hasher = Keccak256::new();
        #[cfg(feature = "cosmwasm")]
        exec_inputs_hasher.update(self.exec_inputs.as_slice());
        #[cfg(not(feature = "cosmwasm"))]
        exec_inputs_hasher.update(BASE64_STANDARD.decode(&self.exec_inputs)?);
        let exec_inputs_hash = exec_inputs_hasher.finalize();

        let mut tally_inputs_hasher = Keccak256::new();
        #[cfg(feature = "cosmwasm")]
        tally_inputs_hasher.update(self.tally_inputs.as_slice());
        #[cfg(not(feature = "cosmwasm"))]
        tally_inputs_hasher.update(BASE64_STANDARD.decode(&self.tally_inputs)?);
        let tally_inputs_hash = tally_inputs_hasher.finalize();

        let mut consensus_filter_hasher = Keccak256::new();
        #[cfg(feature = "cosmwasm")]
        consensus_filter_hasher.update(self.consensus_filter.as_slice());
        #[cfg(not(feature = "cosmwasm"))]
        consensus_filter_hasher.update(BASE64_STANDARD.decode(&self.consensus_filter)?);
        let consensus_filter_hash = consensus_filter_hasher.finalize();

        let mut memo_hasher = Keccak256::new();
        #[cfg(feature = "cosmwasm")]
        memo_hasher.update(self.memo.as_slice());
        #[cfg(not(feature = "cosmwasm"))]
        memo_hasher.update(BASE64_STANDARD.decode(&self.memo)?);
        let memo_hash = memo_hasher.finalize();

        // hash data request
        let mut dr_hasher = Keccak256::new();
        dr_hasher.update(self.version.hash());
        // I don't think we should decode to hash... expensive in cosmwasm no?
        dr_hasher.update(hex::decode(&self.exec_program_id)?);
        dr_hasher.update(exec_inputs_hash);
        dr_hasher.update(self.exec_gas_limit.to_be_bytes());
        dr_hasher.update(hex::decode(&self.tally_program_id)?);
        dr_hasher.update(tally_inputs_hash);
        dr_hasher.update(self.tally_gas_limit.to_be_bytes());
        dr_hasher.update(self.replication_factor.to_be_bytes());
        dr_hasher.update(consensus_filter_hash);
        dr_hasher.update(self.gas_price.to_be_bytes());
        dr_hasher.update(memo_hash);

        Ok(dr_hasher.finalize().into())
    }
}

/// Governance-controlled timeout configuration parameters
#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Deserialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct DrConfig {
    /// Number of blocks after which a data request is timed out while waiting
    /// for commits.
    pub commit_timeout_in_blocks:        NonZero<u8>,
    /// Number of blocks after which a data request is timed out while waiting
    /// for reveals.
    pub reveal_timeout_in_blocks:        NonZero<u8>,
    /// This is the delay before the backup executors are allowed to start
    /// executing the data request.
    pub backup_delay_in_blocks:          NonZero<u8>,
    /// The maximum size of all the reveals in a data request.
    pub dr_reveal_size_limit_in_bytes:   NonZero<u16>,
    /// The maximum size of the input for the execution program.
    pub exec_input_limit_in_bytes:       NonZero<u16>,
    /// The maximum size of the input for the tally program.
    pub tally_input_limit_in_bytes:      NonZero<u16>,
    /// The maximum size of the consensus filter.
    pub consensus_filter_limit_in_bytes: NonZero<u16>,
    /// The maximum size of the memo.
    pub memo_limit_in_bytes:             NonZero<u16>,
    /// The maximum size of the payback address.
    pub payback_address_limit_in_bytes:  NonZero<u16>,
    /// The maximum size of the SEDA payload.
    pub seda_payload_limit_in_bytes:     NonZero<u16>,
}

impl From<DrConfig> for crate::msgs::ExecuteMsg {
    fn from(config: DrConfig) -> Self {
        super::execute::ExecuteMsg::SetDrConfig(config).into()
    }
}

pub type LastSeenIndexKey = (U128, String, String);

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize, Deserialize, Debug, PartialEq))]
pub struct GetDataRequestsByStatusResponse {
    pub is_paused:       bool,
    pub data_requests:   Vec<DataRequestResponse>,
    pub last_seen_index: Option<LastSeenIndexKey>,
    pub total:           u32,
}
