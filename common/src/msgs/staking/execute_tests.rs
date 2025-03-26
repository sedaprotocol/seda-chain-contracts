use serde_json::json;

use super::execute::*;
use crate::msgs;
#[cfg(feature = "cosmwasm")]
use crate::msgs::assert_json_deser;
#[cfg(not(feature = "cosmwasm"))]
use crate::msgs::assert_json_ser;

#[test]
fn json_stake() {
    let serialized_no_memo = json!({
      "stake": {
        "memo": null,
        "proof": "proof",
        "public_key": "public",
      }
    });
    let msg_no_memo: msgs::ExecuteMsg = stake::Execute {
        public_key: "public".to_string(),
        proof:      "proof".to_string(),
        memo:       None,
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg_no_memo, serialized_no_memo);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg_no_memo, serialized_no_memo);

    #[cfg(not(feature = "cosmwasm"))]
    let memo = "memo".to_string();
    #[cfg(feature = "cosmwasm")]
    let memo = "memo".as_bytes().into();

    let serialized_with_memo = json!({
        "stake": {
            "public_key": "public",
            "proof": "proof",
            "memo": memo,
        }
    });
    let msg_with_memo: msgs::ExecuteMsg = stake::Execute {
        public_key: "public".to_string(),
        proof:      "proof".to_string(),
        memo:       Some(memo),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg_with_memo, serialized_with_memo);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg_with_memo, serialized_with_memo);
}

#[test]
fn json_unstake() {
    let serialized = json!({
      "unstake": {
        "proof": "proof",
        "public_key": "public",
      }
    });
    let msg: msgs::ExecuteMsg = unstake::Execute {
        public_key: "public".to_string(),
        proof:      "proof".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, serialized);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, serialized);
}

#[test]
fn json_withdraw() {
    let serialized = json!({
      "withdraw": {
        "proof": "proof",
        "public_key": "public",
        "withdraw_address": "withdraw_address",
      }
    });
    let msg: msgs::ExecuteMsg = withdraw::Execute {
        public_key:       "public".to_string(),
        proof:            "proof".to_string(),
        withdraw_address: "withdraw_address".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, serialized);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, serialized);
}
