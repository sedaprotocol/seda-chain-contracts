use common::state::{DataRequest, DataResult};
use common::types::{Bytes, Hash};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct DataRequestInputs {
    // DR definition
    /// Semantic Version String
    // pub version: Version,
    /// Identifier of DR WASM binary
    pub dr_binary_id: Hash,
    /// Inputs for DR WASM binary
    pub dr_inputs: Bytes,
    /// Identifier of Tally WASM binary
    pub tally_binary_id: Hash,
    /// Inputs for Tally WASM binary
    pub tally_inputs: Bytes,
    /// Amount of required DR executors
    pub replication_factor: u16,
    /// Amount of SEDA tokens per gas unit
    pub gas_price: u128,
    /// Maximum of gas units to be used
    pub gas_limit: u128,
    /// Public info attached to DR
    pub memo: Bytes,

    // Execution Information
    /// Payback address set by the relayer
    pub payback_address: Bytes,
    /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
    pub seda_payload: Bytes,
}

/// Upon posting a data request, it is added to this map with a unique auto-incrementing ID
pub const DATA_REQUESTS: Map<Hash, DataRequest> = Map::new("data_requests_pool");

/// Upon executing a data request, teh result is added to this map with a unique auto-incrementing ID
pub const DATA_RESULTS: Map<Hash, DataResult> = Map::new("data_results_pool");

/// For internal bookkeeping when iterative over pool; index within DR pool
pub const DATA_REQUESTS_POOL_ARRAY: Map<u128, Hash> = Map::new("data_requests_pool_array");

/// A counter for how many data requests are unresolved
pub const DATA_REQUESTS_POOL_COUNT: Item<u128> = Item::new("data_requests_count");

/// Address of the token used for deposit for posting a data request
// TODO: implement deposit for posting data requests
pub const TOKEN: Item<String> = Item::new("token");

/// Address of proxy contract which has permission to set the sender on one's behalf
pub const PROXY_CONTRACT: Item<Addr> = Item::new("proxy_contract");
