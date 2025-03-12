use serde_json::json;

use super::execute::*;
use crate::msgs;
#[cfg(feature = "cosmwasm")]
use crate::msgs::assert_json_deser;
#[cfg(not(feature = "cosmwasm"))]
use crate::msgs::assert_json_ser;

#[test]
fn json_accept_ownership() {
    let expected_json = json!(
    {
        "accept_ownership": {}
    });
    let msg: msgs::ExecuteMsg = accept_ownership::Execute {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_add_to_allowlist() {
    let expected_json = json!({
      "add_to_allowlist": {
        "public_key": "public_key"
      }
    });
    let msg: msgs::ExecuteMsg = add_to_allowlist::Execute {
        public_key: "public_key".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_remove_from_allowlist() {
    let expected_json = json!({
      "remove_from_allowlist": {
        "public_key": "public_key"
      }
    });
    let msg: msgs::ExecuteMsg = remove_from_allowlist::Execute {
        public_key: "public_key".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_transfer_ownership() {
    let expected_json = json!({
      "transfer_ownership": {
        "new_owner": "new_owner"
      }
    });
    let msg: msgs::ExecuteMsg = transfer_ownership::Execute {
        new_owner: "new_owner".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_pause() {
    let expected_json = json!(
    {
        "pause": {}
    });
    let msg: msgs::ExecuteMsg = pause::Execute {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_unpause() {
    let expected_json = json!(
    {
        "unpause": {}
    });
    let msg: msgs::ExecuteMsg = unpause::Execute {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
