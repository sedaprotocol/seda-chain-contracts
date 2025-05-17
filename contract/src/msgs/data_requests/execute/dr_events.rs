use cosmwasm_std::Event;
use seda_common::msgs::data_requests::DrConfig;

use super::CONTRACT_VERSION;

pub fn create_dr_config_event(config: DrConfig) -> Event {
    Event::new("seda-dr-config").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("commit_timeout_in_blocks", config.commit_timeout_in_blocks.to_string()),
        ("reveal_timeout_in_blocks", config.reveal_timeout_in_blocks.to_string()),
        ("backup_delay_in_blocks", config.backup_delay_in_blocks.to_string()),
    ])
}
