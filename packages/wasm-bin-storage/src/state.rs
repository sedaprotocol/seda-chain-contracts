use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct BinaryStruct {
    pub binary: Vec<u8>,
    pub description: String,
}

pub const BINARIES: Map<&u128, BinaryStruct> = Map::new("binaries");
pub const BINARIES_COUNT: Item<u128> = Item::new("binaries_count");
