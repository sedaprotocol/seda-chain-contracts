use alloy_sol_types::sol;
use cosmwasm_std::{StdError, Storage};
use cw_storage_plus::{Item, Map, PrimaryKey};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub type Input = Vec<u8>;
pub type PayloadItem = Vec<u8>;

pub struct EnumerableMap<'a, K, T> {
    pub len: Item<'a, u128>,
    pub items: Map<'a, K, T>,
    pub index_to_key: Map<'a, u128, K>,
    pub key_to_index: Map<'a, K, u128>,
}

#[macro_export]
macro_rules! enumerable_map {
    ($namespace:literal) => {
        EnumerableMap {
            len: Item::new(concat!($namespace, "_len")),
            items: Map::new(concat!($namespace, "_items")),
            index_to_key: Map::new(concat!($namespace, "_index_to_key")),
            key_to_index: Map::new(concat!($namespace, "_key_to_index")),
        }
    };
}

impl<'a, K, T> EnumerableMap<'_, K, T>
where
    T: Serialize + DeserializeOwned,
    K: Serialize + DeserializeOwned + PrimaryKey<'a>,
{
    pub fn initialize(&self, store: &mut dyn Storage) -> Result<(), StdError> {
        self.len.save(store, &0)?;
        Ok(())
    }

    pub fn load(&'a self, store: &dyn Storage, key: K) -> Result<T, StdError> {
        self.items.load(store, key)
    }

    pub fn may_load(&'a self, store: &dyn Storage, key: K) -> Result<Option<T>, StdError> {
        self.items.may_load(store, key)
    }

    pub fn load_at_index(&'a self, store: &dyn Storage, index: u128) -> Result<K, StdError> {
        self.index_to_key.load(store, index)
    }

    pub fn update(&'a self, store: &mut dyn Storage, key: K, item: &T) -> Result<(), StdError> {
        self.items.save(store, key, item)
    }

    pub fn len(&'a self, store: &dyn Storage) -> Result<u128, StdError> {
        self.len.load(store)
    }

    pub fn add(&'a self, store: &mut dyn Storage, key: K, item: T) -> Result<(), StdError> {
        // Error if key already exists
        if self.key_to_index.may_load(store, key.clone())?.is_some() {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len.load(store)?;
        self.items.save(store, key.clone(), &item)?;
        self.index_to_key.save(store, index, &key)?;
        self.key_to_index.save(store, key, &index)?;
        self.len.save(store, &(index + 1))?;
        Ok(())
    }

    pub fn remove(&'a self, store: &mut dyn Storage, key: K) -> Result<(), StdError> {
        let last_index = self.len.load(store)? - 1;
        let last_key = self.index_to_key.load(store, last_index)?;
        let item_index = self.key_to_index.load(store, key.clone())?;

        // Handle special case where item to remove is last item
        if item_index == last_index {
            // Just pop the last item
            self.len.save(store, &last_index)?;
            self.items.remove(store, key);
            return Ok(());
        }

        // Set the index of the item to remove to point to the last item's key
        self.index_to_key.save(store, item_index, &last_key)?;
        // Set the last item's key to use the index of the item to remove
        self.key_to_index.save(store, last_key, &item_index)?;
        // Decrement the length of items by 1
        self.len.save(store, &last_index)?;
        // Remove the item using its key
        self.items.remove(store, key);
        Ok(())
    }
}

// Inputs to hash functions using Solidity ABI encoding
sol! {
    struct DataRequestHashInputs {
        /// Semantic Version
        string version;
        /// Identifier of DR WASM binary
        bytes32 dr_binary_id;
        /// Inputs for DR WASM binary
        bytes dr_inputs;
        /// Maximum of gas units to be used by data request executors
        uint128 gas_limit;
        /// Amount of SEDA tokens per gas unit
        uint128 gas_price;
        /// Maximum of gas units to be used in the tallying process
        uint128 tally_gas_limit;
        /// Public info attached to DR
        bytes memo;
        /// Amount of required DR executors
        uint16 replication_factor;
        /// Identifier of Tally WASM binary
        bytes32 tally_binary_id;
        /// Inputs for Tally WASM binary
        bytes tally_inputs;
    }
    struct DataResultHashInputs {
        /// Semantic Version
        string version;
        /// Identifier of the data request
        bytes32 dr_id;
        /// Block Height at which data request was finalized
        uint128 block_height;
        /// Exit code of Tally WASM binary execution
        uint8 exit_code;
        /// Result from Tally WASM binary execution
        bytes result;
        /// Payback address set by the relayer
        bytes payback_address;
        /// Payload set by SEDA Protocol (e.g. OEV-enabled data requests)
        bytes seda_payload;
    }
}
