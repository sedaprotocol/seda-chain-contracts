use cosmwasm_std::Event;
use seda_common::msgs::data_requests::TimeoutConfig;

use super::CONTRACT_VERSION;

pub fn create_timeout_config_event(config: TimeoutConfig) -> Event {
    Event::new("seda-timeout-config").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("commit_timeout_in_blocks", config.commit_timeout_in_blocks.to_string()),
        ("reveal_timeout_in_blocks", config.reveal_timeout_in_blocks.to_string()),
    ])
}
