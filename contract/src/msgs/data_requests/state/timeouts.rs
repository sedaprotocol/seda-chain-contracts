use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Map;
use seda_common::types::Hash;

pub struct Timeouts<'a> {
    pub timeouts:        Map<'a, (u64, Hash), ()>,
    // Need this so we can remove the timeout by dr_id
    pub hash_to_timeout: Map<'a, &'a Hash, u64>,
}

impl Timeouts<'_> {
    pub fn insert(&self, store: &mut dyn Storage, timeout: u64, dr_id: &Hash) -> StdResult<()> {
        self.hash_to_timeout.save(store, dr_id, &timeout)?;
        self.timeouts.save(store, (timeout, *dr_id), &())?;
        Ok(())
    }

    pub fn remove_by_dr_id(&self, store: &mut dyn Storage, dr_id: &Hash) -> StdResult<()> {
        let timeout_block = self.hash_to_timeout.load(store, dr_id)?;
        self.timeouts.remove(store, (timeout_block, *dr_id));
        self.hash_to_timeout.remove(store, dr_id);
        Ok(())
    }

    pub fn remove_by_timeout_height(&self, store: &mut dyn Storage, timeout_height: u64) -> StdResult<()> {
        // Once the new rust borrow checker is released:
        // It would allow us to to not allocate all the dr_ids for the timeout_height at once.

        // This call allocates all the dr_ids for the timeout_height at once...
        self.get_all_by_timeout_height(store, timeout_height)?
            .into_iter()
            .for_each(|hash| {
                self.timeouts.remove(store, (timeout_height, hash));
                self.hash_to_timeout.remove(store, &hash);
            });

        Ok(())
    }

    pub fn get_timeout_by_dr_id(&self, store: &dyn Storage, dr_id: &Hash) -> StdResult<u64> {
        let timeout_block = self.hash_to_timeout.load(store, dr_id)?;
        Ok(timeout_block)
    }

    // Get all hashes by u64 (timeout)
    pub fn get_all_by_timeout_height(&self, store: &dyn Storage, timeout_height: u64) -> StdResult<Vec<Hash>> {
        let res: StdResult<Vec<_>> = self
            .timeouts
            .prefix(timeout_height)
            .range(store, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| item.map(|(hash, _)| hash))
            .collect();
        res
    }
}
