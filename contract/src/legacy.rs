use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
pub struct TimeoutConfig {
    pub commit_timeout_in_blocks: u64,
    pub reveal_timeout_in_blocks: u64,
}

pub const TIMEOUT_CONFIG: Item<TimeoutConfig> = Item::new("timeout_config");
