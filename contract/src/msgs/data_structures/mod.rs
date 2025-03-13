use std::marker::PhantomData;

use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Item, Map, PrimaryKey};

mod cost_sorted_index;
pub use cost_sorted_index::{CostSorted, CostSortedIndex, Entry};
mod enumerable_set;
pub use enumerable_set::{Enumerable, EnumerableSet};

#[cfg(test)]
mod cost_sorted_index_tests;

pub struct IndexKeyedSet<Value, Key, Kind> {
    pub len:            Item<u32>,
    pub index_to_value: Map<u32, Value>,
    pub key_to_index:   Map<Key, u32>,
    pub kind:           PhantomData<Kind>,
}

impl<'a, Value, Key, Kind> IndexKeyedSet<Value, Key, Kind>
where
    Value: serde::de::DeserializeOwned + serde::Serialize,
    Key: PrimaryKey<'a> + serde::Serialize,
{
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    /// Returns true if the key exists in the set in O(1) time.
    pub fn has(&self, store: &dyn Storage, key: Key) -> bool {
        self.key_to_index.has(store, key)
    }

    /// Returns the length of the set in O(1) time.
    pub fn len(&self, store: &dyn Storage) -> StdResult<u32> {
        self.len.load(store)
    }

    // /// Returns the key at the given index in O(1) time.
    // fn at(&self, store: &dyn Storage, index: u32) -> StdResult<Option<Hash>> {
    //     self.index_to_key.may_load(store, index)
    // }

    /// Returns the index of the key in the set in O(1) time.
    pub fn get_index(&self, store: &dyn Storage, key: Key) -> StdResult<u32> {
        self.key_to_index.load(store, key)
    }
}
