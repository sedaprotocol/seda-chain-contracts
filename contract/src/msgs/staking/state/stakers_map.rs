use super::*;

pub struct StakersMap<'a> {
    pub stakers:     Map<'a, &'a PublicKey, Staker>,
    pub public_keys: EnumerableSet<'a, PublicKey>,
}

impl StakersMap<'_> {
    pub fn initialize(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.public_keys.initialize(store)?;
        Ok(())
    }

    pub fn insert(&self, store: &mut dyn Storage, key: PublicKey, value: &Staker) -> StdResult<()> {
        self.stakers.save(store, &key, value)?;
        self.public_keys.add(store, key)?;
        Ok(())
    }

    pub fn update(&self, store: &mut dyn Storage, key: PublicKey, value: &Staker) -> StdResult<()> {
        self.stakers.save(store, &key, value)?;
        Ok(())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: PublicKey) -> StdResult<()> {
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
        let config = CONFIG.load(store)?;
        if config.allowlist_enabled {
            let allowed = ALLOWLIST.may_load(store, executor)?;
            // If the executor is not in the allowlist, they are not eligible.
            // If the executor is in the allowlist, but the value is false, they are not eligible.
            if allowed.is_none() || !allowed.unwrap() {
                return Ok(false);
            }
        }

        let executor = self.may_get_staker(store, executor)?;
        Ok(match executor {
            Some(staker) => staker.tokens_staked >= config.minimum_stake_for_committee_eligibility,
            None => false,
        })
    }

    pub fn len(&self, store: &dyn Storage) -> StdResult<u32> {
        self.public_keys.len(store)
    }
}

macro_rules! new_stakers_map {
    ($namespace:literal) => {
        StakersMap {
            stakers:     Map::new(concat!($namespace, "_stakers")),
            public_keys: $crate::enumerable_set!(concat!($namespace, "_public_keys")),
        }
    };
}

pub(crate) use new_stakers_map;
use owner::state::ALLOWLIST;
