use super::*;

pub struct DataRequestsMap<'a> {
    pub reqs:       Map<'a, &'a Hash, DataRequest>,
    pub committing: EnumerableSet<'a, Hash>,
    pub revealing:  EnumerableSet<'a, Hash>,
    pub tallying:   EnumerableSet<'a, Hash>,
}

use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Map;

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

    fn add_to_status(&self, store: &mut dyn Storage, key: Hash, status: &DataRequestStatus) -> StdResult<()> {
        match status {
            DataRequestStatus::Committing => self.committing.add(store, key)?,
            DataRequestStatus::Revealing => self.revealing.add(store, key)?,
            DataRequestStatus::Tallying => self.tallying.add(store, key)?,
        }

        Ok(())
    }

    fn remove_from_status(&self, store: &mut dyn Storage, key: Hash, status: &DataRequestStatus) -> StdResult<()> {
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
        key: Hash,
        req: DataRequest,
        status: &DataRequestStatus,
    ) -> StdResult<()> {
        if self.has(store, &key) {
            return Err(StdError::generic_err("Key already exists"));
        }

        self.reqs.save(store, &key, &req)?;
        self.add_to_status(store, key, status)?;

        Ok(())
    }

    fn find_status(&self, store: &dyn Storage, key: Hash) -> StdResult<DataRequestStatus> {
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
        key: Hash,
        dr: DataRequest,
        status: Option<DataRequestStatus>,
    ) -> StdResult<()> {
        // Check if the key exists
        if !self.has(store, &key) {
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
        self.reqs.save(store, &key, &dr)?;
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
    pub fn remove(&self, store: &mut dyn Storage, key: Hash) -> Result<(), StdError> {
        if !self.has(store, &key) {
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
        self.reqs.remove(store, &key);
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

macro_rules! new_enumerable_status_map {
    ($namespace:literal) => {
        DataRequestsMap {
            reqs:       Map::new(concat!($namespace, "_reqs")),
            committing: $crate::enumerable_set!(concat!($namespace, "_committing")),
            revealing:  $crate::enumerable_set!(concat!($namespace, "_revealing")),
            tallying:   $crate::enumerable_set!(concat!($namespace, "_tallying")),
        }
    };
}

pub(super) use new_enumerable_status_map;
