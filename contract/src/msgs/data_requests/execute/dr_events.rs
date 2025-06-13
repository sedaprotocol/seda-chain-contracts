use cosmwasm_std::Event;
use seda_common::msgs::data_requests::DrConfig;

use super::CONTRACT_VERSION;

pub fn create_dr_config_event(config: DrConfig) -> Event {
    Event::new("seda-dr-config").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("commit_timeout_in_blocks", config.commit_timeout_in_blocks.to_string()),
        ("reveal_timeout_in_blocks", config.reveal_timeout_in_blocks.to_string()),
        ("backup_delay_in_blocks", config.backup_delay_in_blocks.to_string()),
        (
            "dr_reveal_size_limit_in_bytes",
            config.dr_reveal_size_limit_in_bytes.to_string(),
        ),
        (
            "exec_input_limit_in_bytes",
            config.exec_input_limit_in_bytes.to_string(),
        ),
        (
            "tally_input_limit_in_bytes",
            config.tally_input_limit_in_bytes.to_string(),
        ),
        (
            "consensus_filter_limit_in_bytes",
            config.consensus_filter_limit_in_bytes.to_string(),
        ),
        ("memo_limit_in_bytes", config.memo_limit_in_bytes.to_string()),
        (
            "payback_address_limit_in_bytes",
            config.payback_address_limit_in_bytes.to_string(),
        ),
        (
            "seda_payload_limit_in_bytes",
            config.seda_payload_limit_in_bytes.to_string(),
        ),
    ])
}
