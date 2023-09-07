use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// Token denomination used for data request executor staking and deposit for posting a data request
pub const TOKEN: Item<String> = Item::new("token");

/// Contract address of `seda-chain-contracts`
pub const SEDA_CHAIN_CONTRACTS: Item<Addr> = Item::new("seda_chain_contracts");
