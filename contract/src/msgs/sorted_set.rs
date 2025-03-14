use seda_common::msgs::data_requests::DataRequest;

use super::*;

pub type IndexKey = (u128, u64, Hash);

pub struct SortedSet {
    pub index:          Map<IndexKey, ()>,
    pub dr_id_to_index: Map<Hash, IndexKey>,
}

impl SortedSet {
    pub fn has(&self, store: &dyn Storage, dr_id: Hash) -> bool {
        self.dr_id_to_index.has(store, dr_id)
    }

    pub fn has_index(&self, store: &dyn Storage, index: IndexKey) -> bool {
        self.index.has(store, index)
    }

    pub fn add(&self, store: &mut dyn Storage, dr_id: Hash, dr: DataRequest) -> StdResult<()> {
        if self.has(store, dr_id) {
            return Err(StdError::generic_err("Key already exists"));
        }

        let gas_price: u128 = dr.gas_price.into();
        let height: u64 = u64::MAX - dr.height;

        let index_key = (gas_price, height, dr_id);
        self.index.save(store, index_key, &())?;

        self.dr_id_to_index.save(store, dr_id, &index_key)?;

        Ok(())
    }

    pub fn add_by_index(&self, store: &mut dyn Storage, index: IndexKey) -> StdResult<()> {
        let hash = index.2;
        if self.has(store, hash) {
            return Err(StdError::generic_err("Key already exists"));
        }

        self.index.save(store, index, &())?;
        self.dr_id_to_index.save(store, hash, &index)?;

        Ok(())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: Hash) -> StdResult<IndexKey> {
        let index = self.dr_id_to_index.load(store, key)?;
        self.index.remove(store, (index.0, index.1, key));
        self.dr_id_to_index.remove(store, key);

        Ok(index)
    }
}

#[macro_export]
macro_rules! sorted_set {
    ($namespace:expr) => {
        SortedSet {
            index:          Map::new(concat!($namespace, "_index")),
            dr_id_to_index: Map::new(concat!($namespace, "_dr_id_to_index")),
        }
    };
}
