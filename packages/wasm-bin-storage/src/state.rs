use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;

#[cw_serde]
pub struct BinaryStruct {
    pub binary: Vec<u8>,
    pub description: String,
}

pub const BINARIES: Map<&String, BinaryStruct> = Map::new("binaries");
