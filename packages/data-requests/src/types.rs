use cosmwasm_std::{StdError, Storage};
use cw_storage_plus::{Item, Map, PrimaryKey};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub type Input = Vec<u8>;
pub type PayloadItem = Vec<u8>;

pub struct EnumerableMap<'a, K, T> {
    len: Item<'a, u128>,
    items: Map<'a, K, T>,
    index_to_key: Map<'a, u128, K>,
    key_to_index: Map<'a, K, u128>,
}

impl<'a, K, T> EnumerableMap<'a, K, T> {
    pub const fn new(
        len_namespace: &'a str,
        items_namespace: &'a str,
        index_to_key_namespace: &'a str,
        key_to_index_namespace: &'a str,
    ) -> Self {
        let len: Item<u128> = Item::new(len_namespace);
        let items: Map<K, T> = Map::new(items_namespace);
        let index_to_key: Map<u128, K> = Map::new(index_to_key_namespace);
        let key_to_index: Map<K, u128> = Map::new(key_to_index_namespace);
        Self {
            len,
            items,
            index_to_key,
            key_to_index,
        }
    }
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
