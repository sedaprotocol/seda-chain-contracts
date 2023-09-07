use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const SEDA_CHAIN_CONTRACTS: Item<Addr> = Item::new("seda_chain_contracts");
