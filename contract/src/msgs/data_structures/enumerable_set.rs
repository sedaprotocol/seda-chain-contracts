use cosmwasm_std::StdError;
use cw_storage_plus::PrimaryKey;

use super::*;

pub struct Enumerable;

pub type EnumerableSet<Key> = IndexKeyedSet<Key, Key, Enumerable>;

impl<'a, Key> IndexKeyedSet<Key, Key, Enumerable>
where
    Key: PrimaryKey<'a> + serde::de::DeserializeOwned + serde::Serialize,
{
    /// Adds a key to the set in O(1) time.
    pub fn add(&self, store: &mut dyn Storage, key: Key) -> StdResult<()> {
        if self.has(store, key.clone()) {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len(store)?;
        self.index_to_value.save(store, index, &key)?;
        self.key_to_index.save(store, key, &index)?;
        self.len.save(store, &(index + 1))?;
        Ok(())
    }

    /// Removes a key from the set in O(1) time.
    pub fn remove(&self, store: &mut dyn Storage, key: Key) -> StdResult<()> {
        let index = self
            .key_to_index
            .may_load(store, key.clone())?
            .ok_or_else(|| StdError::generic_err("Key does not exist"))?;
        let total_items = self.len(store)?;

        // Shouldn't be reachable
        if total_items == 0 {
            unreachable!("No items in the set, so key should not exist");
        }

        // Handle case when removing the last or only item
        // means we can just remove the key and return
        if total_items == 1 || index == total_items - 1 {
            self.index_to_value.remove(store, index);
            self.key_to_index.remove(store, key);
            self.len.save(store, &(total_items - 1))?;
            return Ok(());
        }

        // Swap the last item into the position of the removed item
        let last_index = total_items - 1;
        let last_key = self.index_to_value.load(store, last_index)?;

        // Update mapping for the swapped item
        self.index_to_value.save(store, index, &last_key)?;
        self.key_to_index.save(store, last_key, &index)?;

        // Remove original entries for the removed item
        self.index_to_value.remove(store, last_index);
        self.key_to_index.remove(store, key);

        // Update length
        self.len.save(store, &last_index)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! enumerable_set {
    ($namespace:expr) => {
        $crate::msgs::data_structures::IndexKeyedSet {
            len:            Item::new(concat!($namespace, "_len")),
            index_to_value: Map::new(concat!($namespace, "_index_to_value")),
            key_to_index:   Map::new(concat!($namespace, "_key_to_index")),
            kind:           std::marker::PhantomData::<$crate::msgs::data_structures::Enumerable>,
        }
    };
}
