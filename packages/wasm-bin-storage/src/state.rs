use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cw_storage_plus::Map;

#[cw_serde]
pub struct BinaryStruct {
    pub binary: Binary,
    pub description: String,
}

pub const BINARIES: Map<&String, BinaryStruct> = Map::new("binaries");
