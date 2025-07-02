use cw_storage_plus::{KeyDeserialize, Prefixer, PrimaryKey};
use seda_common::msgs::data_requests::{DataRequestBase, DataRequestContract, DataRequestResponse, LastSeenIndexKey};
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexKey {
    pub gas_price: u128,
    pub height:    u64,
    pub dr_id:     Hash,
}

impl Prefixer<'_> for IndexKey {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        let mut res = self.gas_price.prefix();
        res.extend(self.height.prefix());
        res
    }
}

impl PrimaryKey<'_> for IndexKey {
    type Prefix = (u128, u64);
    type SubPrefix = u128;
    type Suffix = Hash;
    type SuperSuffix = (u64, Hash);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut key = self.gas_price.key();
        key.extend(self.height.key());
        key.extend(self.dr_id.key());
        key
        // <(u128, u64, Hash) as PrimaryKey>::key(&(self.gas_price, self.height,
        // self.dr_id))
    }
}

impl KeyDeserialize for IndexKey {
    type Output = Self;

    const KEY_ELEMS: u16 =
        <u128 as KeyDeserialize>::KEY_ELEMS + <u64 as KeyDeserialize>::KEY_ELEMS + <Hash as KeyDeserialize>::KEY_ELEMS;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let tuple = <(u128, u64, Hash) as KeyDeserialize>::from_vec(value)?;
        Ok(Self {
            gas_price: tuple.0,
            height:    tuple.1,
            dr_id:     tuple.2,
        })
    }
}

impl IndexKey {
    pub fn new<P: Into<u128>>(gas_price: P, height: u64, dr_id: Hash) -> Self {
        Self {
            gas_price: gas_price.into(),
            // The index key is a tuple of (gas_price, height, dr_id)
            // We need to reverse the height to get the correct order.
            // For example, if the height is 1 and 2, the reversed height is u64::MAX - 1 > u64::MAX - 2.
            height: u64::MAX - height,
            dr_id,
        }
    }
}

impl TryFrom<&DataRequestBase> for IndexKey {
    type Error = ContractError;

    fn try_from(value: &DataRequestBase) -> Result<Self, Self::Error> {
        let dr_id = Hash::from_hex_str(&value.id)?;
        Ok(Self::new(value.posted_gas_price, value.height, dr_id))
    }
}

impl TryFrom<&DataRequestContract> for IndexKey {
    type Error = ContractError;

    fn try_from(value: &DataRequestContract) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value.base)
    }
}

impl TryFrom<&DataRequestResponse> for IndexKey {
    type Error = ContractError;

    fn try_from(value: &DataRequestResponse) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value.base)
    }
}

impl TryFrom<LastSeenIndexKey> for IndexKey {
    type Error = ContractError;

    fn try_from(value: LastSeenIndexKey) -> Result<Self, Self::Error> {
        Ok(Self {
            gas_price: value.0.into(),
            height:    u64::MAX - value.1.parse::<u64>().map_err(|_| ContractError::InvalidDrHeight)?,
            dr_id:     Hash::from_hex_str(&value.2)?,
        })
    }
}

impl From<IndexKey> for LastSeenIndexKey {
    fn from(val: IndexKey) -> Self {
        (
            val.gas_price.into(),
            (u64::MAX - val.height).to_string(),
            val.dr_id.to_hex(),
        )
    }
}

/// A structure to store a sorted set of data requests by the `IndexKey`
pub struct SortedSet<'a> {
    pub len:            Item<u32>,
    /// Used to store information about the data request by the `IndexKey` so it
    /// can be sorted
    pub index:          Map<IndexKey, ()>,
    /// Used to store the `IndexKey` by the `Hash` of the data request
    pub dr_id_to_index: Map<&'a Hash, IndexKey>,
}

impl SortedSet<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.len.save(store, &0)?;
        Ok(())
    }

    pub fn len(&self, store: &dyn Storage) -> StdResult<u32> {
        self.len.load(store)
    }

    pub fn has(&self, store: &dyn Storage, dr_id: &Hash) -> bool {
        self.dr_id_to_index.has(store, dr_id)
    }

    pub fn has_index(&self, store: &dyn Storage, index: IndexKey) -> bool {
        self.index.has(store, index)
    }

    pub fn add(&self, store: &mut dyn Storage, dr_id: &Hash, dr: DataRequestContract) -> StdResult<()> {
        if self.has(store, dr_id) {
            return Err(StdError::generic_err("Key already exists"));
        }

        let index_key = IndexKey::try_from(&dr)?;
        self.dr_id_to_index.save(store, dr_id, &index_key)?;
        self.index.save(store, index_key, &())?;

        let len = self.len(store)?;
        self.len.save(store, &(len + 1))?;

        Ok(())
    }

    pub fn add_by_index(&self, store: &mut dyn Storage, index: IndexKey) -> StdResult<()> {
        let hash = &index.dr_id;
        if self.has(store, hash) {
            return Err(StdError::generic_err("Key already exists"));
        }

        self.index.save(store, index, &())?;
        self.dr_id_to_index.save(store, hash, &index)?;

        let len = self.len(store)?;
        self.len.save(store, &(len + 1))?;

        Ok(())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<IndexKey> {
        let index = self.dr_id_to_index.load(store, key)?;
        self.index.remove(store, index);
        self.dr_id_to_index.remove(store, key);

        let len = self.len(store)?;
        self.len.save(store, &(len - 1))?;

        Ok(index)
    }
}

#[macro_export]
macro_rules! sorted_set {
    ($namespace:expr) => {
        SortedSet {
            len:            Item::new(concat!($namespace, "_len")),
            index:          Map::new(concat!($namespace, "_index")),
            dr_id_to_index: Map::new(concat!($namespace, "_dr_id_to_index")),
        }
    };
}
