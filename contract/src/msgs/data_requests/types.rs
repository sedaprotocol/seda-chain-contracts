use cw_storage_plus::PrimaryKey;
use serde::{de::DeserializeOwned, Serialize};

use super::*;

pub struct EnumerableMap<'a, K, T> {
    pub len:          Item<'a, u128>,
    pub items:        Map<'a, K, T>,
    pub index_to_key: Map<'a, u128, K>,
    pub key_to_index: Map<'a, K, u128>,
}

#[macro_export]
macro_rules! enumerable_map {
    ($namespace:literal) => {
        EnumerableMap {
            len:          Item::new(concat!($namespace, "_len")),
            items:        Map::new(concat!($namespace, "_items")),
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

    pub fn update(&'a self, store: &mut dyn Storage, key: K, item: &T) -> Result<(), StdError> {
        if self.key_to_index.may_load(store, key.clone())?.is_none() {
            return Err(StdError::generic_err("Key does not exist"));
        }
        self.items.save(store, key, item)
    }

    pub fn len(&'a self, store: &dyn Storage) -> Result<u128, StdError> {
        self.len.load(store)
    }

    pub fn get_by_key(&'a self, store: &dyn Storage, key: K) -> Result<Option<T>, StdError> {
        self.items.may_load(store, key)
    }

    pub fn get_by_index(&'a self, store: &dyn Storage, index: u128) -> Result<Option<T>, StdError> {
        let Some(key) = self.index_to_key.may_load(store, index)? else {
            return Ok(None);
        };
        self.items.may_load(store, key)
    }

    pub fn insert(&'a self, store: &mut dyn Storage, key: K, item: T) -> Result<(), StdError> {
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

    /// Removes an item from the map by key.
    /// Swaps the last item with the item to remove.
    /// Then pops the last item.
    pub fn swap_remove(&'a self, store: &mut dyn Storage, key: K) -> Result<(), StdError> {
        // check if the key exists
        if self.key_to_index.may_load(store, key.clone())?.is_none() {
            return Err(StdError::generic_err("Key does not exist"));
        }

        let last_index = self.len.load(store)? - 1;
        let last_key = self.index_to_key.load(store, last_index)?;
        let item_index = self.key_to_index.load(store, key.clone())?;

        // swap the last item with the item to remove in the index to key
        self.index_to_key.save(store, item_index, &last_key)?;
        // update the key to index for the last key
        self.key_to_index.save(store, last_key.clone(), &item_index)?;
        // remove the last index
        self.index_to_key.remove(store, last_index);
        // remove the key asked for removal
        self.items.remove(store, key.clone());
        self.key_to_index.remove(store, key);

        // Decrement the length of items by 1
        self.len.save(store, &last_index)?;
        Ok(())
    }
}
