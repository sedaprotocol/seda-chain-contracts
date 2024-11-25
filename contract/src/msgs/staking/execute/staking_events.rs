use cosmwasm_std::{Event, Uint128};
use seda_common::msgs::staking::{Staker, StakingConfig};

use super::CONTRACT_VERSION;

pub(in crate::msgs::staking::execute) fn create_executor_event(staker: Staker, public_key: String) -> Event {
    Event::new("seda-executor").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("identity", public_key),
        ("tokens_staked", staker.tokens_staked.to_string()),
        (
            "tokens_pending_withdrawal",
            staker.tokens_pending_withdrawal.to_string(),
        ),
        ("memo", staker.memo.map(|m| m.to_base64()).unwrap_or_default()),
    ])
}

pub(in crate::msgs::staking::execute) fn create_executor_action_event(
    action: &str,
    public_key: String,
    sender: String,
    amount: Uint128,
    seq: Uint128,
) -> Event {
    Event::new("seda-executor-action").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("action", action.to_string()),
        ("identity", public_key),
        ("sender", sender),
        ("amount", amount.to_string()),
        ("seq", seq.to_string()),
    ])
}

pub fn create_staking_config_event(config: StakingConfig) -> Event {
    Event::new("seda-staking-config").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        (
            "minimum_stake_for_committee_eligibility",
            config.minimum_stake_for_committee_eligibility.to_string(),
        ),
        (
            "minimum_stake_to_register",
            config.minimum_stake_to_register.to_string(),
        ),
        ("allowlist_enabled", config.allowlist_enabled.to_string()),
    ])
}
