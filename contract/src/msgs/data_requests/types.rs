use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Key, PrimaryKey};

use super::*;

#[cw_serde]
pub struct StatusIndexKey {
    pub index:  u128,
    pub status: DataRequestStatus,
}

impl StatusIndexKey {
    pub fn new(index: u128, status: Option<DataRequestStatus>) -> Self {
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
    type Suffix = u128;
    type SuperSuffix = ();

    fn key(&self) -> Vec<Key> {
        let mut keys = self.status.key();
        keys.push(Key::Val128(self.index.to_be_bytes()));
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

pub struct EnumerableMap<'a> {
    pub len:            Item<'a, u128>,
    pub reqs:           Map<'a, &'a Hash, StatusValue>,
    pub index_to_key:   Map<'a, u128, Hash>,
    pub key_to_index:   Map<'a, &'a Hash, u128>,
    pub status_to_keys: Map<'a, &'a StatusIndexKey, Hash>,
}

#[macro_export]
macro_rules! enumerable_map {
    ($namespace:literal) => {
        EnumerableMap {
            len:            Item::new(concat!($namespace, "_len")),
            reqs:           Map::new(concat!($namespace, "_reqs")),
            index_to_key:   Map::new(concat!($namespace, "_index_to_key")),
            key_to_index:   Map::new(concat!($namespace, "_key_to_index")),
            status_to_keys: Map::new(concat!($namespace, "_status_to_keys")),
        }
    };
}

impl<'a> EnumerableMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    pub fn update(&'a self, store: &mut dyn Storage, key: &Hash, req: &StatusValue) -> StdResult<()> {
        if self.key_to_index.may_load(store, key)?.is_none() {
            return Err(StdError::generic_err("Key does not exist"));
        }
        self.reqs.save(store, key, req)
    }

    pub fn len(&'a self, store: &dyn Storage) -> Result<u128, StdError> {
        self.len.load(store)
    }

    pub fn get_by_key(&'a self, store: &dyn Storage, key: &Hash) -> StdResult<Option<DataRequest>> {
        self.reqs.may_load(store, key).map(|opt| opt.map(|req| req.req))
    }

    pub fn get_by_index(&'a self, store: &dyn Storage, index: u128) -> StdResult<Option<DataRequest>> {
        if let Some(key) = self.index_to_key.may_load(store, index)? {
            self.get_by_key(store, &key)
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, store: &mut dyn Storage, key: &Hash, req: &StatusValue) -> StdResult<()> {
        if self.key_to_index.may_load(store, key)?.is_some() {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index = self.len.load(store)?;
        self.reqs.save(store, key, req)?;
        self.index_to_key.save(store, index, key)?;
        self.key_to_index.save(store, key, &index)?;
        self.status_to_keys.save(
            store,
            &StatusIndexKey {
                status: req.status.clone(),
                index,
            },
            key,
        )?;
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
    ) -> StdResult<Vec<DataRequest>> {
        let mut requests = vec![];
        let keys = self
            .status_to_keys
            .prefix(status)
            .range(store, None, None, Order::Ascending);

        for key in keys {
            let (_, hash) = key?;
            let req = self.reqs.load(store, &hash)?;
            requests.push(req.req);
        }

        Ok(requests)
    }
}
