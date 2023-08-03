use cw_storage_plus::{Map, Item};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

// TODO in a real world, we would have semvar on this struct as well.
#[cw_serde]
pub struct BinaryStruct {
    // TODO unsure if this type is good for binaries or we should use a base Vec<u8>
    pub binary: cosmwasm_std::Binary,
    pub description: String,
}

pub const BINARIES: Map<&str, BinaryStruct> = Map::new("binaries");
pub const CONFIG: Item<Config> = Item::new("config");
