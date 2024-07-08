use cosmwasm_std::Storage;
use cw_storage_plus::Bound;

use super::*;

// Map DrID -> DataRequest
// EnumrableSet of DrID's per status.

pub struct EnumerableSet<'a> {
    pub len:          Item<'a, u32>,
    pub key_to_index: Map<'a, &'a Hash, u32>,
    pub index_to_key: Map<'a, u32, Hash>,
}

#[macro_export]
macro_rules! enumerable_set {
    ($namespace:expr) => {
        EnumerableSet {
            len:          Item::new(concat!($namespace, "_len")),
            key_to_index: Map::new(concat!($namespace, "_key_to_index")),
            index_to_key: Map::new(concat!($namespace, "_index_to_key")),
        }
    };
}

impl EnumerableSet<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    /// Returns true if the key exists in the set in O(1) time.
    fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
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
    fn add(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<()> {
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
    fn remove(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<()> {
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

pub struct DataRequestsMap<'a> {
    pub reqs:       Map<'a, &'a Hash, DataRequest>,
    pub committing: EnumerableSet<'a>,
    pub revealing:  EnumerableSet<'a>,
    pub tallying:   EnumerableSet<'a>,
}

#[macro_export]
macro_rules! enumerable_status_map {
    ($namespace:literal) => {
        DataRequestsMap {
            reqs:       Map::new(concat!($namespace, "_reqs")),
            committing: $crate::enumerable_set!(concat!($namespace, "_committing")),
            revealing:  $crate::enumerable_set!(concat!($namespace, "_revealing")),
            tallying:   $crate::enumerable_set!(concat!($namespace, "_tallying")),
        }
    };
}

impl DataRequestsMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.committing.initialize(store)?;
        self.revealing.initialize(store)?;
        self.tallying.initialize(store)?;
        Ok(())
    }

    pub fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
        self.reqs.has(store, key)
    }

    fn add_to_status(&self, store: &mut dyn Storage, key: &Hash, status: &DataRequestStatus) -> StdResult<()> {
        match status {
            DataRequestStatus::Committing => self.committing.add(store, key)?,
            DataRequestStatus::Revealing => self.revealing.add(store, key)?,
            DataRequestStatus::Tallying => self.tallying.add(store, key)?,
        }

        Ok(())
    }

    fn remove_from_status(&self, store: &mut dyn Storage, key: &Hash, status: &DataRequestStatus) -> StdResult<()> {
        match status {
            DataRequestStatus::Committing => self.committing.remove(store, key)?,
            DataRequestStatus::Revealing => self.revealing.remove(store, key)?,
            DataRequestStatus::Tallying => self.tallying.remove(store, key)?,
        }

        Ok(())
    }

    pub fn insert(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        req: DataRequest,
        status: &DataRequestStatus,
    ) -> StdResult<()> {
        if self.has(store, key) {
            return Err(StdError::generic_err("Key already exists"));
        }

        self.reqs.save(store, key, &req)?;
        self.add_to_status(store, key, status)?;

        Ok(())
    }

    fn find_status(&self, store: &dyn Storage, key: &Hash) -> StdResult<DataRequestStatus> {
        if self.committing.has(store, key) {
            return Ok(DataRequestStatus::Committing);
        }

        if self.revealing.has(store, key) {
            return Ok(DataRequestStatus::Revealing);
        }

        if self.tallying.has(store, key) {
            return Ok(DataRequestStatus::Tallying);
        }

        Err(StdError::generic_err("Key does not exist"))
    }

    pub fn update(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        dr: DataRequest,
        status: Option<DataRequestStatus>,
    ) -> StdResult<()> {
        // Check if the key exists
        if !self.has(store, key) {
            return Err(StdError::generic_err("Key does not exist"));
        }

        // If we need to update the status, we need to remove the key from the current status
        if let Some(status) = status {
            // Grab the current status.
            let current_status = self.find_status(store, key)?;
            // world view = we should only update from committing -> revealing -> tallying.
            // Either the concept is fundamentally flawed or the implementation is wrong.
            match current_status {
                DataRequestStatus::Committing => {
                    assert_eq!(
                        status,
                        DataRequestStatus::Revealing,
                        "Cannot update a request status from committing to anything other than revealing"
                    )
                }
                DataRequestStatus::Revealing => {
                    assert_eq!(
                        status,
                        DataRequestStatus::Tallying,
                        "Cannot update a request status from revealing to anything other than tallying"
                    );
                }
                DataRequestStatus::Tallying => {
                    assert_ne!(
                        current_status,
                        DataRequestStatus::Tallying,
                        "Cannot update a request's status that is tallying"
                    );
                }
            }

            // remove from current status, then add to new one.
            self.remove_from_status(store, key, &current_status)?;
            self.add_to_status(store, key, &status)?;
        }

        // always update the request
        self.reqs.save(store, key, &dr)?;
        Ok(())
    }

    pub fn may_get(&self, store: &dyn Storage, key: &Hash) -> StdResult<Option<DataRequest>> {
        self.reqs.may_load(store, key)
    }

    pub fn get(&self, store: &dyn Storage, key: &Hash) -> StdResult<DataRequest> {
        self.reqs.load(store, key)
    }

    /// Removes an req from the map by key.
    /// Swaps the last req with the req to remove.
    /// Then pops the last req.
    pub fn remove(&self, store: &mut dyn Storage, key: &Hash) -> Result<(), StdError> {
        if !self.has(store, key) {
            return Err(StdError::generic_err("Key does not exist"));
        }

        // world view = we only remove a data request that is done tallying.
        // Either the concept is fundamentally flawed or the implementation is wrong.
        let current_status = self.find_status(store, key)?;
        assert_eq!(
            current_status,
            DataRequestStatus::Tallying,
            "Cannot remove a request that is not tallying"
        );

        // remove the request
        self.reqs.remove(store, key);
        // remove from the status
        self.remove_from_status(store, key, &current_status)?;

        Ok(())
    }

    pub fn get_requests_by_status(
        &self,
        store: &dyn Storage,
        status: &DataRequestStatus,
        offset: u32,
        limit: u32,
    ) -> StdResult<Vec<DataRequest>> {
        let start = Some(Bound::inclusive(offset));
        let end = Some(Bound::exclusive(offset + limit));
        let requests = match status {
            DataRequestStatus::Committing => &self.committing,
            DataRequestStatus::Revealing => &self.revealing,
            DataRequestStatus::Tallying => &self.tallying,
        }
        .index_to_key
        .range(store, start, end, Order::Ascending)
        .flat_map(|result| result.map(|(_, key)| self.reqs.load(store, &key)))
        .collect::<StdResult<Vec<_>>>()?;

        Ok(requests)
    }
}
