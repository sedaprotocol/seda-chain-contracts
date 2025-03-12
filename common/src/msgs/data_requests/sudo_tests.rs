use std::collections::HashMap;

use serde_json::json;

use super::sudo::*;
use crate::msgs;
#[cfg(feature = "cosmwasm")]
use crate::msgs::assert_json_deser;
#[cfg(not(feature = "cosmwasm"))]
use crate::msgs::assert_json_ser;

#[test]
fn json_remove_requests() {
    let expected_json = json!({
    "remove_data_requests": {
        "requests": {
            "dr_id1": [
                    {
                        "burn": {
                            "amount": "100"
                        }
                    },
                    {
                        "executor_reward": {
                            "amount": "100",
                            "identity": "identity"
                        }
                    },
                    {
                        "data_proxy_reward": {
                            "amount": "100",
                            "payout_address": "payout_address"
                        },
                    }
                ],
        },
    }
    });
    let mut requests = HashMap::new();
    requests.insert(
        "dr_id1".to_string(),
        vec![
            DistributionMessage::Burn(DistributionBurn { amount: 100u128.into() }),
            DistributionMessage::ExecutorReward(DistributionExecutorReward {
                amount:   100u128.into(),
                identity: "identity".to_string(),
            }),
            DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                amount:         100u128.into(),
                payout_address: "payout_address".to_string(),
            }),
        ],
    );

    let msg: msgs::SudoMsg = remove_requests::Sudo { requests }.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}

#[test]
fn json_remove_timed_out_data_requests() {
    let expected_json = json!({"expire_data_requests": {}});
    let msg: msgs::SudoMsg = expire_data_requests::Sudo {}.into();
    #[cfg(not(feature = "cosmwasm"))]
    assert_json_ser(msg, expected_json);
    #[cfg(feature = "cosmwasm")]
    assert_json_deser(msg, expected_json);
}
