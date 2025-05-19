use super::*;
use crate::msgs::sorted_set::IndexKey;

pub struct DataRequestsMap<'a> {
    pub reqs:       Map<&'a Hash, DataRequest>,
    pub committing: SortedSet<'a>,
    pub revealing:  SortedSet<'a>,
    pub tallying:   SortedSet<'a>,
    pub timeouts:   Timeouts<'a>,
}

use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Map;

impl DataRequestsMap<'_> {
    pub fn initialize(&self, _store: &mut dyn Storage) -> StdResult<()> {
        self.committing.initialize(_store)?;
        self.revealing.initialize(_store)?;
        self.tallying.initialize(_store)?;
        Ok(())
    }

    pub fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
        self.reqs.has(store, key)
    }

    fn move_to_status(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        current_status: &DataRequestStatus,
        new_status: &DataRequestStatus,
    ) -> StdResult<()> {
        let index = match current_status {
            DataRequestStatus::Committing => self.committing.remove(store, key)?,
            DataRequestStatus::Revealing => self.revealing.remove(store, key)?,
            DataRequestStatus::Tallying => self.tallying.remove(store, key)?,
        };

        match new_status {
            DataRequestStatus::Committing => self.committing.add_by_index(store, index)?,
            DataRequestStatus::Revealing => self.revealing.add_by_index(store, index)?,
            DataRequestStatus::Tallying => self.tallying.add_by_index(store, index)?,
        };

        Ok(())
    }

    fn add_to_status(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        req: DataRequest,
        status: &DataRequestStatus,
    ) -> StdResult<()> {
        match status {
            DataRequestStatus::Committing => self.committing.add(store, key, req)?,
            DataRequestStatus::Revealing => self.revealing.add(store, key, req)?,
            DataRequestStatus::Tallying => self.tallying.add(store, key, req)?,
        };

        Ok(())
    }

    fn remove_from_status(&self, store: &mut dyn Storage, key: &Hash, status: &DataRequestStatus) -> StdResult<()> {
        match status {
            DataRequestStatus::Committing => self.committing.remove(store, key)?,
            DataRequestStatus::Revealing => self.revealing.remove(store, key)?,
            DataRequestStatus::Tallying => self.tallying.remove(store, key)?,
        };

        Ok(())
    }

    pub fn insert(
        &self,
        store: &mut dyn Storage,
        current_height: u64,
        key: &Hash,
        req: DataRequest,
        status: &DataRequestStatus,
    ) -> StdResult<()> {
        if self.has(store, key) {
            return Err(StdError::generic_err("Key already exists"));
        }

        self.reqs.save(store, key, &req)?;
        self.add_to_status(store, key, req, status)?;
        let dr_config = DR_CONFIG.load(store)?;
        self.timeouts
            .insert(store, current_height + dr_config.commit_timeout_in_blocks, key)?;

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
        current_height: u64,
        timeout: bool,
    ) -> StdResult<()> {
        // Check if the key exists
        if !self.has(store, key) {
            return Err(StdError::generic_err("Key does not exist"));
        }

        // If we need to update the status, we need to remove the key from the current
        // status
        if let Some(new_status) = status {
            // Grab the current status.
            let current_status = self.find_status(store, key)?;
            // world view = we should only update from committing -> revealing -> tallying,
            // or if it's a timeout, from any status to tallying.
            // Either the concept is fundamentally flawed or the implementation is wrong.
            match &current_status {
                _ if timeout => {
                    assert_eq!(
                        new_status,
                        DataRequestStatus::Tallying,
                        "Cannot update a timed out request status to anything other than tallying"
                    );
                }
                DataRequestStatus::Committing => {
                    assert_eq!(
                        new_status,
                        DataRequestStatus::Revealing,
                        "Cannot update a request status from committing to anything other than revealing"
                    );

                    // We change the timeout to the reveal timeout when commit -> reveal
                    self.timeouts.remove_by_dr_id(store, key)?;
                    let dr_config = DR_CONFIG.load(store)?;
                    self.timeouts
                        .insert(store, dr_config.reveal_timeout_in_blocks + current_height, key)?;
                }
                DataRequestStatus::Revealing => {
                    assert_eq!(
                        new_status,
                        DataRequestStatus::Tallying,
                        "Cannot update a request status from revealing to anything other than tallying"
                    );

                    // We remove the timeout when reveal -> tally
                    self.timeouts.remove_by_dr_id(store, key)?;
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
            self.move_to_status(store, key, &current_status, &new_status)?;
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
        last_seen_index: Option<IndexKey>,
        limit: u32,
    ) -> StdResult<(Vec<DataRequest>, Option<IndexKey>, u32)> {
        let start = last_seen_index.map(Bound::exclusive);

        let set = match status {
            DataRequestStatus::Committing => &self.committing,
            DataRequestStatus::Revealing => &self.revealing,
            DataRequestStatus::Tallying => &self.tallying,
        };
        let set_len = set.len(store)?;

        if last_seen_index.is_some_and(|index| !set.has_index(store, index)) {
            return Ok((vec![], None, set_len));
        }

        let requests = set
            .index
            // Start is the max argument since we're ordering descending
            .range(store, None, start, Order::Descending)
            .take(limit as usize)
            .flat_map(|result| result.map(|(key, _)| self.reqs.load(store, &key.dr_id)))
            .collect::<StdResult<Vec<_>>>()?;

        if requests.len() < limit as usize {
            return Ok((requests, None, set_len));
        }

        // The last seen index is the last element in the list
        let new_last_seen_index = requests.last().map(IndexKey::try_from).transpose()?;

        Ok((requests, new_last_seen_index, set_len))
    }

    pub fn expire_data_requests(&self, store: &mut dyn Storage, current_height: u64) -> StdResult<Vec<String>> {
        // remove them from the timeouts and return the hashes
        let drs_to_update_to_tally = self.timeouts.remove_by_timeout_height(store, current_height)?;

        drs_to_update_to_tally
            .into_iter()
            .map(|hash| {
                // get the dr itself
                let dr = self.get(store, &hash)?;
                // update it to tallying
                self.update(
                    store,
                    &hash,
                    dr,
                    Some(DataRequestStatus::Tallying),
                    current_height,
                    true,
                )?;
                Ok(hash.to_hex())
            })
            .collect::<StdResult<Vec<_>>>()
    }
}

macro_rules! new_enumerable_status_map {
    ($namespace:literal) => {
        DataRequestsMap {
            reqs:       Map::new(concat!($namespace, "_reqs")),
            committing: $crate::sorted_set!(concat!($namespace, "_committing")),
            revealing:  $crate::sorted_set!(concat!($namespace, "_revealing")),
            tallying:   $crate::sorted_set!(concat!($namespace, "_tallying")),
            timeouts:   Timeouts {
                timeouts:        Map::new(concat!($namespace, "_timeouts")),
                hash_to_timeout: Map::new(concat!($namespace, "_hash_to_timeout")),
            },
        }
    };
}

pub(super) use new_enumerable_status_map;
