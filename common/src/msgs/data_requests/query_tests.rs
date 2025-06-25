use serde_json::json;

use super::{query::QueryMsg as DrQueryMsg, DataRequestStatus};
use crate::{
    msgs::*,
    types::{ToHexStr, U128},
};

#[test]
fn json_get_data_request() {
    let expected_json = json!({
      "get_data_request": {
        "dr_id": "dr_id"
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequest {
        dr_id: "dr_id".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_request_commitment() {
    let expected_json = json!({
      "get_data_request_commitment": {
        "dr_id": "dr_id",
        "public_key": "public_key",
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestCommitment {
        dr_id:      "dr_id".to_string(),
        public_key: "public_key".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_request_commitments() {
    let expected_json = json!({
        "get_data_request_commitments": {
            "dr_id": "dr_id",
        }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestCommitments {
        dr_id: "dr_id".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_request_reveal() {
    let expected_json = json!({
      "get_data_request_reveal": {
        "dr_id": "dr_id",
        "public_key": "public_key",
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestReveal {
        dr_id:      "dr_id".to_string(),
        public_key: "public_key".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_request_reveals() {
    let expected_json = json!({
      "get_data_request_reveals": {
        "dr_id": "dr_id",
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestReveals {
        dr_id: "dr_id".to_string(),
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_requests_by_status_no_last_seen() {
    let expected_json = json!({
      "get_data_requests_by_status": {
        "status": "committing",
        "last_seen_index": null,
        "limit": 10,
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestsByStatus {
        status:          DataRequestStatus::Committing,
        last_seen_index: None,
        limit:           10,
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_requests_by_status_with_last_seen() {
    let u128_max: U128 = u128::MAX.into();
    let expected_json = json!({
      "get_data_requests_by_status": {
        "status": "committing",
        "last_seen_index": Some((u128_max, u64::MAX.to_string(), [0; 32].to_hex())),
        "limit": u32::MAX,
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestsByStatus {
        status:          DataRequestStatus::Committing,
        last_seen_index: Some((u128_max, u64::MAX.to_string(), [0; 32].to_hex())),
        limit:           u32::MAX,
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_get_data_requests_statuses() {
    let expected_json = json!({
      "get_data_requests_statuses": {
        "dr_ids": ["dr_id1", "dr_id2"],
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestsStatuses {
        dr_ids: vec!["dr_id1".to_string(), "dr_id2".to_string()],
    }
    .into();

    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
