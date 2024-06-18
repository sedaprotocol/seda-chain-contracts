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
            reqs:           Map::new(concat!($namespace, "_reqs")),
            index_to_key:   Map::new(concat!($namespace, "_index_to_key")),
            key_to_index:   Map::new(concat!($namespace, "_key_to_index")),
            status_to_keys: Map::new(concat!($namespace, "_status_to_keys")),
        }
    };
}

impl<'a> DataRequestsMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    pub fn has(&self, store: &dyn Storage, key: &Hash) -> bool {
        self.key_to_index.has(store, key)
    }

    pub fn update(
        &'a self,
        store: &mut dyn Storage,
        key: &Hash,
        dr: DataRequest,
        status: Option<DataRequestStatus>,
    ) -> StdResult<()> {
        let Some(index) = self.key_to_index.may_load(store, key)? else {
            return Err(StdError::generic_err("Key does not exist"));
        };

        // remove the old status key
        let old = self.reqs.load(store, key)?;
        let old_status_key = StatusIndexKey::new(index, Some(old.status.clone()));
        let status = if let Some(status) = status {
            self.status_to_keys.remove(store, &old_status_key);
            self.status_to_keys
                .save(store, &StatusIndexKey::new(index, Some(status.clone())), key)?;
            status
        } else {
            old.status
        };
        self.reqs.save(store, key, &StatusValue::with_status(dr, status))
    }

    pub fn len(&'a self, store: &dyn Storage) -> Result<u32, StdError> {
        self.len.load(store)
    }

    pub fn may_get_by_key(&'a self, store: &dyn Storage, key: &Hash) -> StdResult<Option<DataRequest>> {
        self.reqs.may_load(store, key).map(|opt| opt.map(|req| req.req))
    }

    pub fn get_by_key(&'a self, store: &dyn Storage, key: &Hash) -> StdResult<DataRequest> {
        self.reqs.load(store, key).map(|req| req.req)
    }

    pub fn may_get_by_index(&'a self, store: &dyn Storage, index: u32) -> StdResult<Option<DataRequest>> {
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
        self.reqs.save(store, key, &StatusValue::new(req))?;
        self.index_to_key.save(store, index, key)?;
        self.key_to_index.save(store, key, &index)?;
        self.status_to_keys
            .save(store, &StatusIndexKey::new(index, None), key)?;
        self.len.save(store, &(index + 1))?;
        Ok(())
    }

    /// Removes an req from the map by key.
    /// Swaps the last req with the req to remove.
    /// Then pops the last req.
    pub fn swap_remove(&self, store: &mut dyn Storage, key: &Hash) -> Result<(), StdError> {
        let req_index = match self.key_to_index.may_load(store, key)? {
            Some(index) => index,
            None => return Err(StdError::generic_err("Key does not exist")),
        };

        let last_index = self.len.load(store)? - 1;
        let last_key = self.index_to_key.load(store, last_index)?;
        let StatusValue {
            req: _,
            status: last_status,
        } = self.reqs.load(store, &last_key)?;

        // swap the last item with the item to remove in the index to key
        self.index_to_key.save(store, req_index, &last_key)?;
        self.status_to_keys.save(
            store,
            &StatusIndexKey {
                status: last_status.clone(),
                index:  req_index,
            },
            &last_key,
        )?;
        // update the key to index for the last key
        self.key_to_index.save(store, &last_key, &req_index)?;
        // remove the last index
        self.index_to_key.remove(store, last_index);
        self.status_to_keys.remove(
            store,
            &StatusIndexKey {
                status: last_status,
                index:  last_index,
            },
        );
        // remove the key asked for removal
        self.reqs.remove(store, key);
        self.key_to_index.remove(store, key);

        // Decrement the length of items by 1
        self.len.save(store, &last_index)?;
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
