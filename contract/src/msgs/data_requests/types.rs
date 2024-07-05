use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Bound, Key, PrimaryKey};

use super::*;

#[cw_serde]
pub struct StatusIndexKey {
    pub index:  u32,
    pub status: DataRequestStatus,
}

impl StatusIndexKey {
    pub fn new(index: u32, status: Option<DataRequestStatus>) -> Self {
        Self {
            index,
            status: status.unwrap_or(DataRequestStatus::Committing),
        }
    }
}

// Implement PrimaryKey for StatusIndexKey
impl<'a> PrimaryKey<'a> for &'a StatusIndexKey {
    type Prefix = DataRequestStatus;
    type SubPrefix = ();
    type Suffix = u32;
    type SuperSuffix = ();

    fn key(&self) -> Vec<Key> {
        let mut keys = self.status.key();
        keys.push(Key::Val32(self.index.to_be_bytes()));
        keys
    }
}

#[cw_serde]
pub struct StatusValue {
    pub req:    DataRequest,
    pub status: DataRequestStatus,
}

impl StatusValue {
    pub fn new(req: DataRequest) -> Self {
        Self {
            req,
            status: DataRequestStatus::Committing,
        }
    }

    pub fn with_status(req: DataRequest, status: DataRequestStatus) -> Self {
        Self { req, status }
    }
}

pub struct DataRequestsMap<'a> {
    pub len:            Item<'a, u32>,
    pub committing_len: Item<'a, u32>,
    pub revealing_len:  Item<'a, u32>,
    pub tallying_len:   Item<'a, u32>,
    pub reqs:           Map<'a, &'a Hash, StatusValue>,
    pub index_to_key:   Map<'a, u32, Hash>,
    pub key_to_index:   Map<'a, &'a Hash, u32>,
    pub status_to_keys: Map<'a, &'a StatusIndexKey, Hash>,
}

#[macro_export]
macro_rules! enumerable_status_map {
    ($namespace:literal) => {
        DataRequestsMap {
            len:            Item::new(concat!($namespace, "_len")),
            committing_len: Item::new(concat!($namespace, "_committing_len")),
            revealing_len:  Item::new(concat!($namespace, "_revealing_len")),
            tallying_len:   Item::new(concat!($namespace, "_tallying_len")),
            reqs:           Map::new(concat!($namespace, "_reqs")),
            index_to_key:   Map::new(concat!($namespace, "_index_to_key")),
            key_to_index:   Map::new(concat!($namespace, "_key_to_index")),
            status_to_keys: Map::new(concat!($namespace, "_status_to_keys")),
        }
    };
}

impl DataRequestsMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        self.committing_len.save(store, &0)?;
        self.revealing_len.save(store, &0)?;
        self.tallying_len.save(store, &0)?;
        Ok(())
    }

    pub fn get_status_len_item(&self, status: &DataRequestStatus) -> &Item<u32> {
        match status {
            DataRequestStatus::Committing => &self.committing_len,
            DataRequestStatus::Revealing => &self.revealing_len,
            DataRequestStatus::Tallying => &self.tallying_len,
        }
    }

    pub fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
        self.key_to_index.has(store, key)
    }

    pub fn update(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        dr: DataRequest,
        status: Option<DataRequestStatus>,
    ) -> StdResult<()> {
        let Some(old) = self.reqs.may_load(store, key)? else {
            return Err(StdError::generic_err("Key does not exist"));
        };

        let status = status.unwrap_or(old.status.clone());

        self.swap_remove(store, key)?;
        self.insert_with_status(store, key, dr, status)?;
        Ok(())
    }

    pub fn len(&self, store: &dyn Storage) -> Result<u32, StdError> {
        self.len.load(store)
    }

    pub fn may_get_by_key(&self, store: &dyn Storage, key: &Hash) -> StdResult<Option<DataRequest>> {
        self.reqs.may_load(store, key).map(|opt| opt.map(|req| req.req))
    }

    pub fn get_by_key(&self, store: &dyn Storage, key: &Hash) -> StdResult<DataRequest> {
        self.reqs.load(store, key).map(|req| req.req)
    }

    pub fn may_get_by_index(&self, store: &dyn Storage, index: u32) -> StdResult<Option<DataRequest>> {
        if let Some(key) = self.index_to_key.may_load(store, index)? {
            self.may_get_by_key(store, &key)
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, store: &mut dyn Storage, key: &Hash, req: DataRequest) -> StdResult<()> {
        if self.key_to_index.may_load(store, key)?.is_some() {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len.load(store)?;
        let status_len_item = self.get_status_len_item(&DataRequestStatus::Committing);
        let status_index = status_len_item.load(store)?;

        // save the request
        self.reqs.save(store, key, &StatusValue::new(req))?;
        self.index_to_key.save(store, index, key)?;
        self.key_to_index.save(store, key, &index)?;
        self.status_to_keys
            .save(store, &StatusIndexKey::new(status_index, None), key)?;

        // increment the indexes
        self.len.save(store, &(index + 1))?;
        status_len_item.save(store, &(status_index + 1))?;
        Ok(())
    }

    // only to be internally used by update
    fn insert_with_status(
        &self,
        store: &mut dyn Storage,
        key: &Hash,
        req: DataRequest,
        status: DataRequestStatus,
    ) -> StdResult<()> {
        if self.key_to_index.may_load(store, key)?.is_some() {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len.load(store)?;
        let status_len_item = self.get_status_len_item(&status);
        let status_index = status_len_item.load(store)?;

        // Correctly save the new status key
        self.reqs
            .save(store, key, &StatusValue::with_status(req, status.clone()))?;
        self.index_to_key.save(store, index, key)?;
        self.key_to_index.save(store, key, &index)?;
        self.status_to_keys
            .save(store, &StatusIndexKey::new(status_index, Some(status)), key)?;

        // Increment the indexes
        self.len.save(store, &(index + 1))?;
        status_len_item.save(store, &(status_index + 1))?;
        Ok(())
    }

    /// Removes an req from the map by key.
    /// Swaps the last req with the req to remove.
    /// Then pops the last req.
    pub fn swap_remove(&self, store: &mut dyn Storage, key: &Hash) -> Result<(), StdError> {
        // Load the (index, status) of the req to remove
        let req_index = self
            .key_to_index
            .may_load(store, key)?
            .ok_or_else(|| StdError::generic_err("Key does not exist"))?;
        let req_status = self.reqs.load(store, key)?.status;

        // Get the current length
        let total_items = self.len.load(store)?;

        // Shouldn't be reachable
        if total_items == 0 {
            unreachable!("No items in the map, so key should not exist");
        }

        let status_len_item = self.get_status_len_item(&req_status);
        let req_status_index = status_len_item.load(store)? - 1;

        // Handle case when removing the last or only item
        // means we can just remove the key and return
        if total_items == 1 || req_index == total_items - 1 {
            self.index_to_key.remove(store, req_index);
            self.key_to_index.remove(store, key);
            self.reqs.remove(store, key);
            self.status_to_keys.remove(
                store,
                &StatusIndexKey {
                    status: req_status,
                    index:  req_status_index,
                },
            );
            self.len.save(store, &(total_items - 1))?;
            status_len_item.save(store, &req_status_index)?;
            return Ok(());
        }

        // Swap the last item into the position of the removed item
        let last_index = total_items - 1;
        let last_key = self.index_to_key.load(store, last_index)?;
        let last_status = self.reqs.load(store, &last_key)?.status;
        let last_status_len_item = self.get_status_len_item(&last_status);
        let last_status_index = last_status_len_item.load(store)? - 1;

        // Update mapping for the swapped item
        self.index_to_key.save(store, req_index, &last_key)?;
        self.key_to_index.save(store, &last_key, &req_index)?;

        // Update status_to_keys mapping
        // in a block to make reading the code easier
        {
            // Remove the entry for the last element from the status_to_keys map.
            // This is necessary because the last element will either be moved to a new index.
            self.status_to_keys.remove(
                store,
                &StatusIndexKey {
                    status: last_status.clone(),
                    index:  last_status_index,
                },
            );
            // Remove the entry for the element that is being removed from the status_to_keys map.
            self.status_to_keys.remove(
                store,
                &StatusIndexKey {
                    status: req_status,
                    index:  req_status_index,
                },
            );
            // Add a new entry for the element that was previously last in the status_to_keys map at its new index.
            self.status_to_keys.save(
                store,
                &StatusIndexKey {
                    status: last_status,
                    index:  req_index,
                },
                &last_key,
            )?;
        }

        // Remove original entries for the removed item
        self.index_to_key.remove(store, last_index);
        self.key_to_index.remove(store, key);
        self.reqs.remove(store, key);

        // Update length
        self.len.save(store, &last_index)?;
        status_len_item.save(store, &req_status_index)?;

        Ok(())
    }

    pub fn get_requests_by_status(
        &self,
        store: &dyn Storage,
        status: DataRequestStatus,
        offset: u32,
        limit: u32,
    ) -> StdResult<Vec<DataRequest>> {
        let start = Bound::inclusive(offset);
        let end = Bound::exclusive(offset + limit);
        let requests = self
            .status_to_keys
            .prefix(status)
            .range(store, Some(start), Some(end), Order::Ascending)
            .map(|key| {
                let (_, key) = key?;
                self.reqs.load(store, &key).map(|req| req.req)
            })
            .collect::<StdResult<Vec<_>>>()?;

        Ok(requests)
    }
}
