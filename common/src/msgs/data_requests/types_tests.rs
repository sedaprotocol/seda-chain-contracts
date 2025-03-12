use std::collections::HashMap;

use serde_json::json;

use super::{DataRequest, PostDataRequestArgs, RevealBody, TimeoutConfig};
#[cfg(feature = "cosmwasm")]
use crate::msgs::assert_json_deser;
use crate::{msgs::assert_json_ser, types::*};

#[derive(Debug, serde::Deserialize)]
struct DRIdTestCase {
    request_id: String,
    args:       PostDataRequestArgs,
}

#[test]
fn data_request_id_vector() {
    let test_vector = include_str!("dr_id.test_vector.json");
    let cases: Vec<DRIdTestCase> = serde_json::from_str(test_vector).unwrap();

    cases.into_iter().for_each(|case| {
        let dr_id = case.args.try_hash().unwrap();
        assert_eq!(case.request_id, dr_id.to_hex());
    });
}

#[test]
fn json_data_request() {
    let id = "id".to_string();
    let version = "1.0.0".to_string();
    let exec_program_id = "exec_program_id".to_string();
    #[cfg(not(feature = "cosmwasm"))]
    let exec_inputs = "exec_inputs".to_string();
    #[cfg(feature = "cosmwasm")]
    let exec_inputs: Bytes = "exec_inputs".as_bytes().into();
    let exec_gas_limit = 1;
    let tally_program_id = "tally_program_id".to_string();
    #[cfg(not(feature = "cosmwasm"))]
    let tally_inputs = "tally_inputs".to_string();
    #[cfg(feature = "cosmwasm")]
    let tally_inputs: Bytes = "tally_inputs".as_bytes().into();
    let tally_gas_limit = 1;
    let replication_factor = 1;
    #[cfg(not(feature = "cosmwasm"))]
    let consensus_filter = "consensus_filter".to_string();
    #[cfg(feature = "cosmwasm")]
    let consensus_filter: Bytes = "consensus_filter".as_bytes().into();
    let gas_price: U128 = 1u128.into();
    #[cfg(not(feature = "cosmwasm"))]
    let memo = "memo".to_string();
    #[cfg(feature = "cosmwasm")]
    let memo: Bytes = "memo".as_bytes().into();
    #[cfg(not(feature = "cosmwasm"))]
    let payback_address = "payback_address".to_string();
    #[cfg(feature = "cosmwasm")]
    let payback_address: Bytes = "payback_address".as_bytes().into();
    #[cfg(not(feature = "cosmwasm"))]
    let seda_payload = "seda_payload".to_string();
    #[cfg(feature = "cosmwasm")]
    let seda_payload: Bytes = "seda_payload".as_bytes().into();
    let commits = HashMap::from([("key".to_string(), "value".hash())]);
    let reveals = HashMap::new();
    let height = 1;

    let expected_json = json!({
      "id": id,
      "version": version,
      "exec_program_id": exec_program_id,
      "exec_inputs": exec_inputs,
      "exec_gas_limit": exec_gas_limit,
      "tally_program_id": tally_program_id,
      "tally_inputs": tally_inputs,
      "tally_gas_limit": tally_gas_limit,
      "replication_factor": replication_factor,
      "consensus_filter": consensus_filter,
      "gas_price": gas_price,
      "memo": memo,
      "payback_address": payback_address,
      "seda_payload": seda_payload,
      "commits": commits,
      "reveals": {},
      "height": height,
    });

    let msg = DataRequest {
        id,
        version: version.parse().unwrap(),
        exec_program_id,
        exec_inputs,
        exec_gas_limit,
        tally_program_id,
        tally_inputs,
        tally_gas_limit,
        replication_factor,
        consensus_filter,
        gas_price,
        memo,
        payback_address,
        seda_payload,
        commits,
        reveals,
        height,
    };

    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_reveal_body() {
    let id = "id".to_string();
    let salt = "salt".to_string();
    let exit_code = 0;
    let gas_used = 1;
    #[cfg(not(feature = "cosmwasm"))]
    let reveal = "reveal".to_string();
    #[cfg(feature = "cosmwasm")]
    let reveal: Bytes = "reveal".as_bytes().into();
    let proxy_public_keys = vec!["key1".to_string(), "key2".to_string()];

    let expected_json = json!({
      "id": id,
      "salt": salt,
      "exit_code": exit_code,
      "gas_used": gas_used,
      "reveal": reveal,
      "proxy_public_keys": proxy_public_keys,
    });

    let msg = RevealBody {
        id,
        salt,
        exit_code,
        gas_used,
        reveal,
        proxy_public_keys,
    };

    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_post_data_request_args() {
    let version = "1.0.0".to_string();
    let exec_program_id = "exec_program_id".to_string();
    #[cfg(not(feature = "cosmwasm"))]
    let exec_inputs = "exec_inputs".to_string();
    #[cfg(feature = "cosmwasm")]
    let exec_inputs: Bytes = "exec_inputs".as_bytes().into();
    let exec_gas_limit = 1;
    let tally_program_id = "tally_program_id".to_string();
    #[cfg(not(feature = "cosmwasm"))]
    let tally_inputs = "tally_inputs".to_string();
    #[cfg(feature = "cosmwasm")]
    let tally_inputs: Bytes = "tally_inputs".as_bytes().into();
    let tally_gas_limit = 1;
    let replication_factor = 1;
    #[cfg(not(feature = "cosmwasm"))]
    let consensus_filter = "consensus_filter".to_string();
    #[cfg(feature = "cosmwasm")]
    let consensus_filter: Bytes = "consensus_filter".as_bytes().into();
    let gas_price: U128 = 1u128.into();
    #[cfg(not(feature = "cosmwasm"))]
    let memo = "memo".to_string();
    #[cfg(feature = "cosmwasm")]
    let memo: Bytes = "memo".as_bytes().into();

    let expected_json = json!({
        "version": version,
        "exec_program_id": exec_program_id,
        "exec_inputs": exec_inputs,
        "exec_gas_limit": exec_gas_limit,
        "tally_program_id": tally_program_id,
        "tally_inputs": tally_inputs,
        "tally_gas_limit": tally_gas_limit,
        "replication_factor": replication_factor,
        "consensus_filter": consensus_filter,
        "gas_price": gas_price,
        "memo": memo,
    });

    let msg = PostDataRequestArgs {
        version: version.parse().unwrap(),
        exec_program_id,
        exec_inputs,
        exec_gas_limit,
        tally_program_id,
        tally_inputs,
        tally_gas_limit,
        replication_factor,
        consensus_filter,
        gas_price,
        memo,
    };

    assert_json_ser(msg, expected_json);
}

#[test]
fn json_timeout_config() {
    let expected_json = json!({
        "commit_timeout_in_blocks": 5,
        "reveal_timeout_in_blocks": 10,
    });

    let msg = TimeoutConfig {
        commit_timeout_in_blocks: 5,
        reveal_timeout_in_blocks: 10,
    };

    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
