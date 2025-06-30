use std::num::NonZero;

use cosmwasm_std::Uint128;
use seda_common::msgs::data_requests::DrConfig;

// 10_000 SEDA
pub const INITIAL_MINIMUM_STAKE: Uint128 = Uint128::new(10_000_000_000_000_000_000_000);

pub const INITIAL_DR_CONFIG: DrConfig = DrConfig {
    commit_timeout_in_blocks:        NonZero::new(50).unwrap(),
    reveal_timeout_in_blocks:        NonZero::new(5).unwrap(),
    backup_delay_in_blocks:          NonZero::new(5).unwrap(),
    // 24 KB
    dr_reveal_size_limit_in_bytes:   NonZero::new(24_000).unwrap(),
    // 2 KB,
    exec_input_limit_in_bytes:       NonZero::new(2_048).unwrap(),
    // 512 B
    tally_input_limit_in_bytes:      NonZero::new(512).unwrap(),
    // 512 B
    consensus_filter_limit_in_bytes: NonZero::new(512).unwrap(),
    // 512 B
    memo_limit_in_bytes:             NonZero::new(512).unwrap(),
    // 128 B
    payback_address_limit_in_bytes:  NonZero::new(128).unwrap(),
    // 512 B
    seda_payload_limit_in_bytes:     NonZero::new(512).unwrap(),
};
