use serde_json::json;

use super::{query::QueryMsg as DrQueryMsg, DataRequestStatus};
use crate::msgs::*;

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
fn json_get_data_requests_by_status() {
    let expected_json = json!({
      "get_data_requests_by_status": {
        "status": "committing",
        "offset": 0,
        "limit": 10,
      }
    });
    let msg: QueryMsg = DrQueryMsg::GetDataRequestsByStatus {
        status: DataRequestStatus::Committing,
        offset: 0,
        limit:  10,
    }
    .into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
