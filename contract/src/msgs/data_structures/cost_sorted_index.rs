use std::borrow::Cow;

use cosmwasm_std::{StdResult, Storage, Uint128};
use seda_common::types::Hash;
use serde::{Deserialize, Serialize};

use super::IndexKeyedSet;

/// A struct that represents an entry in the cost-sorted index.
/// It contains a cost and a dr id.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Clone, Debug, PartialEq, Eq))]
pub struct Entry<'a> {
    pub cost: Uint128,
    /// The hash of the data request.
    /// A cow is used to avoid cloning the key when it is already owned.
    pub hash: Cow<'a, Hash>,
}

impl Entry<'_> {
    pub fn new(cost: Uint128, key: Hash) -> Self {
        Self {
            cost,
            hash: Cow::Owned(key),
        }
    }
}

/// A zero-sized type used to mark the `IndexKeyedSet` as a cost-sorted index.
pub struct CostSorted;
/// A type alias for an `IndexKeyedSet` that is a cost-sorted index.
pub type CostSortedIndex<'a> = IndexKeyedSet<Entry<'a>, &'a Hash, CostSorted>;

// TODO: could improve this with a binary search...
impl<'a> IndexKeyedSet<Entry<'a>, &'a Hash, CostSorted> {
    /// Adds an entry to the cost-sorted index in O(n) time.
    /// The entry is inserted in descending order of cost.
    /// If two entries have the same cost, the new entry is appended making it a
    /// LILO (Last In Last Out) data structure.
    pub fn add(&self, store: &mut dyn Storage, entry: Entry<'_>) -> StdResult<()> {
        let len = self.len(store)?;
        let mut pos = len;
        // In descending order, insert before the first entry with a lower cost.
        // Equal costs will not trigger this condition, so new entries are appended.
        for i in 0..len {
            let existing_entry = self.index_to_value.load(store, i)?;
            if entry.cost > existing_entry.cost {
                pos = i;
                break;
            }
        }
        // Shift entries right to make room.
        for i in (pos..len).rev() {
            let old_entry = self.index_to_value.load(store, i)?;
            self.index_to_value.save(store, i + 1, &old_entry)?;
            self.key_to_index.save(store, &old_entry.hash, &(i + 1))?;
        }
        self.index_to_value.save(store, pos, &entry)?;
        self.key_to_index.save(store, &entry.hash, &pos)?;
        self.len.save(store, &(len + 1))?;
        Ok(())
    }

    /// Removes an entry from the cost-sorted index in O(n) time.
    pub fn remove(&self, store: &mut dyn Storage, key: &Hash) -> StdResult<()> {
        let pos = self.key_to_index.load(store, key)?;
        let len = self.len(store)?;
        self.index_to_value.remove(store, pos);
        self.key_to_index.remove(store, key);
        // Shift entries left to fill the gap.
        for i in pos + 1..len {
            let entry = self.index_to_value.load(store, i)?;
            self.index_to_value.save(store, i - 1, &entry)?;
            self.key_to_index.save(store, &entry.hash, &(i - 1))?;
        }
        self.index_to_value.remove(store, len - 1);
        self.len.save(store, &(len - 1))?;
        Ok(())
    }
}

#[macro_export]
macro_rules! cost_sorted_index {
    ($namespace:expr) => {
        $crate::msgs::data_structures::IndexKeyedSet {
            len:            Item::new(concat!($namespace, "_len")),
            index_to_value: Map::new(concat!($namespace, "_index_to_value")),
            key_to_index:   Map::new(concat!($namespace, "_key_to_index")),
            kind:           std::marker::PhantomData::<$crate::msgs::data_structures::CostSorted>,
        }
    };
}
