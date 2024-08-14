use std::rc::Rc;

use seda_common::msgs::staking::{Staker, StakingConfig};

use super::*;

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

pub struct StakersMap<'a> {
    pub stakers:     Map<'a, &'a PublicKey, Staker>,
    pub public_keys: EnumerableSet<'a, PublicKey>,
}

impl StakersMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.public_keys.initialize(store)?;
        Ok(())
    }

    pub fn insert(&self, store: &mut dyn Storage, key: Rc<PublicKey>, value: &Staker) -> StdResult<()> {
        self.stakers.save(store, &key, value)?;
        self.public_keys.add(store, key.clone())?;
        Ok(())
    }

    pub fn update(&self, store: &mut dyn Storage, key: Rc<PublicKey>, value: &Staker) -> StdResult<()> {
        self.stakers.save(store, &key, value)?;
        Ok(())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: Rc<PublicKey>) -> StdResult<()> {
        self.stakers.remove(store, &key);
        self.public_keys.remove(store, key)?;
        Ok(())
    }

    pub fn may_get_staker(&self, store: &dyn Storage, pub_key: &PublicKey) -> StdResult<Option<Staker>> {
        self.stakers.may_load(store, pub_key)
    }

    pub fn get_staker(&self, store: &dyn Storage, pub_key: &PublicKey) -> StdResult<Staker> {
        self.stakers.load(store, pub_key)
    }

    pub fn is_executor_eligible(&self, store: &dyn Storage, executor: &PublicKey) -> StdResult<bool> {
        let executor = self.may_get_staker(store, executor)?;
        let value = match executor {
            Some(staker) => staker.tokens_staked >= CONFIG.load(store)?.minimum_stake_for_committee_eligibility,
            None => false,
        };

        Ok(value)
    }

    pub fn len(&self, store: &dyn Storage) -> StdResult<u32> {
        self.public_keys.len(store)
    }
}

macro_rules! stakers_map {
    ($namespace:literal) => {
        StakersMap {
            stakers:     Map::new(concat!($namespace, "_stakers")),
            public_keys: $crate::enumerable_set!(concat!($namespace, "_public_keys")),
        }
    };
}

/// A map of stakers (of address to info).
pub const STAKERS: StakersMap = stakers_map!("data_request_executors");
