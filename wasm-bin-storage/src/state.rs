use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct BinaryStruct {
    // TODO unsure if this type is good for binaries or we should use a base Vec<u8>
    pub binary: cosmwasm_std::Binary,
    pub description: String,
}

pub const BINARIES: Map<&u128, BinaryStruct> = Map::new("binaries");
pub const BINARIES_COUNT: Item<u128> = Item::new("binaries_count");
