use super::*;

pub struct EnumerableSet<'a> {
    pub len:          Item<'a, u32>,
    pub key_to_index: Map<'a, &'a Hash, u32>,
    pub index_to_key: Map<'a, u32, Hash>,
}

impl EnumerableSet<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    /// Returns true if the key exists in the set in O(1) time.
    pub fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
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

    /// Adds a key to the set in O(1) time.
    pub fn add(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<()> {
        if self.has(store, key) {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len(store)?;
        self.key_to_index.save(store, key, &index)?;
        self.index_to_key.save(store, index, key)?;
        self.len.save(store, &(index + 1))?;
        Ok(())
    }

    /// Removes a key from the set in O(1) time.
    pub fn remove(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<()> {
        let index = self
            .key_to_index
            .may_load(store, key)?
            .ok_or_else(|| StdError::generic_err("Key does not exist"))?;
        let total_items = self.len(store)?;

        // Shouldn't be reachable
        if total_items == 0 {
            unreachable!("No items in the set, so key should not exist");
        }

        // Handle case when removing the last or only item
        // means we can just remove the key and return
        if total_items == 1 || index == total_items - 1 {
            self.index_to_key.remove(store, index);
            self.key_to_index.remove(store, key);
            self.len.save(store, &(total_items - 1))?;
            return Ok(());
        }

        // Swap the last item into the position of the removed item
        let last_index = total_items - 1;
        let last_key = self.index_to_key.load(store, last_index)?;

        // Update mapping for the swapped item
        self.index_to_key.save(store, index, &last_key)?;
        self.key_to_index.save(store, &last_key, &index)?;

        // Remove original entries for the removed item
        self.index_to_key.remove(store, last_index);
        self.key_to_index.remove(store, key);

        // Update length
        self.len.save(store, &last_index)?;
        Ok(())
    }
}