use serde_json::json;

use super::{Staker, StakingConfig};
use crate::msgs::*;

#[test]
fn json_staker() {
    let serialized_with_no_memo = json!({
      "memo": null,
      "tokens_staked": "100",
      "tokens_pending_withdrawal": "0",
    });
    let staker_with_no_memo = Staker {
        memo:                      None,
        tokens_staked:             100u128.into(),
        tokens_pending_withdrawal: 0u128.into(),
    };

    assert_json_deser(staker_with_no_memo, serialized_with_no_memo);

    #[cfg(not(feature = "cosmwasm"))]
    let memo = "memo".to_string();
    #[cfg(feature = "cosmwasm")]
    let memo = "memo".as_bytes().into();

    let serialized_with_memo = json!({
      "memo": memo,
      "tokens_staked": "100",
      "tokens_pending_withdrawal": "0",
    });
    let staker_with_memo = Staker {
        memo:                      Some(memo),
        tokens_staked:             100u128.into(),
        tokens_pending_withdrawal: 0u128.into(),
    };

    assert_json_deser(staker_with_memo, serialized_with_memo);
}

#[test]
fn json_staking_config() {
    let expected_json = json!({
      "minimum_stake_to_register": "100",
      "minimum_stake_for_committee_eligibility": "100",
      "allowlist_enabled": true,
    });
    let msg = StakingConfig {
        minimum_stake_to_register:               100u128.into(),
        minimum_stake_for_committee_eligibility: 100u128.into(),
        allowlist_enabled:                       true,
    };

    assert_json_deser(msg, expected_json);
}

#[test]
fn json_staker_and_seq() {
    let serialized_with_no_staker = json!({
      "staker": null,
      "seq": "100",
    });
    let staker_and_seq_with_no_staker = super::StakerAndSeq {
        staker: None,
        seq:    100u128.into(),
    };

    assert_json_deser(staker_and_seq_with_no_staker, serialized_with_no_staker);

    #[cfg(not(feature = "cosmwasm"))]
    let memo = "memo".to_string();
    #[cfg(feature = "cosmwasm")]
    let memo = "memo".as_bytes().into();

    let serialized_with_staker = json!({
      "staker": {
        "memo": memo,
        "tokens_staked": "100",
        "tokens_pending_withdrawal": "0",
      },
      "seq": "100",
    });
    let staker_and_seq_with_staker = super::StakerAndSeq {
        staker: Some(Staker {
            memo:                      Some(memo),
            tokens_staked:             100u128.into(),
            tokens_pending_withdrawal: 0u128.into(),
        }),
        seq:    100u128.into(),
    };

    assert_json_deser(staker_and_seq_with_staker, serialized_with_staker);
}
