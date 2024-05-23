
use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::types::{Bytes, Commitment, Memo, Secp256k1PublicKey, Hash};

/// Token denom used for staking (e.g., `aseda`).
pub const TOKEN: Item<String> = Item::new("token");

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// Address of staking contract owner.
pub const OWNER: Item<Addr> = Item::new("owner");

/// Address of pending staking contract owner.
pub const PENDING_OWNER: Item<Option<Addr>> = Item::new("pending_owner");

/// Allowlist of public keys that can register as a staker.
pub const ALLOWLIST: Map<&Secp256k1PublicKey, bool> = Map::new("allowlist");

/// A map of stakers (of address to info).
pub const STAKERS: Map<&Secp256k1PublicKey, Staker> = Map::new("data_request_executors");


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

/// A data request executor with staking info and optional p2p multi address
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct Staker {
    pub memo:                      Option<String>,
    pub tokens_staked:             u128,
    pub tokens_pending_withdrawal: u128,
}


/// Governance-controlled configuration parameters
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct StakingConfig {
    /// Minimum amount of SEDA tokens required to register as a data request executor
    pub minimum_stake_to_register:               u128,
    /// Minimum amount of SEDA tokens required to be eligible for committee inclusion
    pub minimum_stake_for_committee_eligibility: u128,
    /// Whether the allowlist is enabled
    pub allowlist_enabled:                       bool,
}