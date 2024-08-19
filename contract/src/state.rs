use cosmwasm_std::{StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

use crate::types::PublicKey;

/// Token denom used for staking (e.g., `aseda`).
pub const TOKEN: Item<String> = Item::new("token");

/// Chain ID of the network (e.g., `seda-1`).
/// Used as a "magic number"
pub const CHAIN_ID: Item<String> = Item::new("chain_id");

/// A map of public key to account sequence number.
const ACCOUNT_SEQ: Map<&PublicKey, u128> = Map::new("account_seq");

pub fn get_seq(store: &dyn Storage, public_key: &PublicKey) -> StdResult<u128> {
    ACCOUNT_SEQ.may_load(store, public_key).map(|x| x.unwrap_or_default())
}

pub fn inc_get_seq(store: &mut dyn Storage, public_key: &PublicKey) -> StdResult<Uint128> {
    let seq = ACCOUNT_SEQ.may_load(store, public_key)?.unwrap_or_default();
    ACCOUNT_SEQ.save(store, public_key, &(seq + 1))?;
    Ok(seq.into())
}
