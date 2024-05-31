use super::*;

/// Governance-controlled configuration parameters.
pub const CONFIG: Item<StakingConfig> = Item::new("config");

/// A map of stakers (of address to info).
pub const STAKERS: Map<&PublicKey, Staker> = Map::new("data_request_executors");

/// A map of public key to account sequence number.
const ACCOUNT_SEQ: Map<&PublicKey, u128> = Map::new("account_seq");

pub fn get_seq(store: &dyn Storage, public_key: &PublicKey) -> StdResult<u128> {
    ACCOUNT_SEQ.may_load(store, public_key).map(|x| x.unwrap_or_default())
}

pub fn inc_get_seq(store: &mut dyn Storage, public_key: &PublicKey) -> StdResult<u128> {
    let seq = ACCOUNT_SEQ.may_load(store, public_key)?.unwrap_or_default();
    ACCOUNT_SEQ.save(store, public_key, &(seq + 1))?;
    Ok(seq)
}
