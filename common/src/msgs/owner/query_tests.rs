use serde_json::json;

use super::query::QueryMsg as OwnerQueryMsg;
use crate::msgs;
#[cfg(feature = "cosmwasm")]
use crate::msgs::assert_json_deser;
#[cfg(not(feature = "cosmwasm"))]
use crate::msgs::assert_json_ser;

#[test]
fn json_get_owner() {
    let expected_json = json!(
    {
      "get_owner": {}
    });
    let msg: msgs::QueryMsg = OwnerQueryMsg::GetOwner {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_pending_owner() {
    let expected_json = json!(
    {
        "get_pending_owner": {}
    });
    let msg: msgs::QueryMsg = OwnerQueryMsg::GetPendingOwner {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_is_paused() {
    let expected_json = json!(
    {
        "is_paused": {}
    });
    let msg: msgs::QueryMsg = OwnerQueryMsg::IsPaused {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
