use std::collections::HashMap;

use seda_common::{
    msgs::data_requests::{
        sudo::{DistributionBurn, DistributionDataProxyReward, DistributionExecutorReward, DistributionMessage},
        DataRequestStatus,
        RevealBody,
    },
    types::{HashSelf, TryHashSelf},
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
fn remove_data_request() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();
    let executor = test_info.new_executor("exec", Some(51), Some(1));

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();

    // owner removes a data result
    // reward goes to executor
    // invalid identities and address are burned
    // non staked executor is not rewarded
    // remainder refunds to alice
    let bob = test_info.new_executor("bob", Some(2), None);
    test_info
        .remove_data_request(
            dr_id,
            vec![
                DistributionMessage::Burn(DistributionBurn { amount: 1u128.into() }),
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    payout_address: executor.addr().to_string(),
                    amount:         5u128.into(),
                }),
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: executor.pub_key_hex(),
                    amount:   5u128.into(),
                }),
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    amount:         2u128.into(),
                    payout_address: "invalid".to_string(),
                }),
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: "invalid".to_string(),
                    amount:   2u128.into(),
                }),
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: bob.pub_key_hex(),
                    amount:   2u128.into(),
                }),
            ],
        )
        .unwrap();
    assert_eq!(55, test_info.executor_balance("exec"));
    assert_eq!(4, test_info.executor_balance("alice"));
    assert_eq!(2, test_info.executor_balance("bob"));

    // get the staker info for the executor
    let staker = test_info.get_staker(executor.pub_key()).unwrap();
    assert_eq!(5, staker.tokens_pending_withdrawal.u128());
}

#[test]
fn remove_data_request_retains_order() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();
    let executor = test_info.new_executor("exec", Some(51), Some(1));

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();

    // owner removes a data result
    // reward goes to executor
    // remainder refunds to alice
    test_info
        .remove_data_request(
            dr_id,
            vec![
                DistributionMessage::Burn(DistributionBurn { amount: 10u128.into() }),
                DistributionMessage::Burn(DistributionBurn { amount: 8u128.into() }),
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    payout_address: executor.addr().to_string(),
                    amount:         3u128.into(),
                }),
            ],
        )
        .unwrap();
    // it's 52 since there should only be enough to reward 2 after the burn messages.
    // this also tests that the order of the messages is retained
    assert_eq!(52, test_info.executor_balance("exec"));
    assert_eq!(1, test_info.executor_balance("alice"));
}

#[test]
fn remove_data_requests() {
    let mut test_info = TestInfo::init();

    // post data request 1
    let mut alice = test_info.new_executor("alice", Some(42), Some(1));
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id1 = test_info
        .post_data_request(&mut alice, dr1, vec![], vec![], 1, None)
        .unwrap();

    // alice commits data result 1
    let alice_reveal1 = RevealBody {
        id:                dr_id1.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id1, alice_reveal1.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&alice, &dr_id1, alice_reveal1.clone()).unwrap();

    // post data request 2
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id2 = test_info
        .post_data_request(&mut alice, dr2, vec![], vec![], 2, None)
        .unwrap();

    // alice commits data result 2
    let alice_reveal2 = RevealBody {
        id:                dr_id2.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id2, alice_reveal2.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&alice, &dr_id2, alice_reveal2.clone()).unwrap();

    // owner posts data results
    let mut to_remove = HashMap::new();
    to_remove.insert(
        dr_id1.clone(),
        vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
            amount:   10u128.into(),
            identity: alice.pub_key_hex(),
        })],
    );
    to_remove.insert(
        dr_id2.clone(),
        vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
            amount:   10u128.into(),
            identity: alice.pub_key_hex(),
        })],
    );
    let removed = test_info.remove_data_requests(to_remove).unwrap();
    removed.iter().for_each(|r| assert_eq!(0, r.1));
}

#[test]
fn remove_data_request_invalid_status_codes() {
    let mut test_info = TestInfo::init();

    // remove a dr with an invalid dr_id and dr that does not exist
    let mut to_remove = HashMap::new();
    to_remove.insert("does_not_exist".to_string(), vec![]);
    to_remove.insert(
        "2404059f879876ad51abe32ad9099d5fe4085c473d54571f109d637a25d62885".to_string(),
        vec![],
    );
    let removed = test_info.remove_data_requests(to_remove).unwrap();
    // test this way since tests are not in wasm so hashmap order is non deterministic
    assert_eq!(1, removed.iter().find(|r| r.0 == "does_not_exist").unwrap().1);
    assert_eq!(
        2,
        removed
            .iter()
            .find(|r| r.0 == "2404059f879876ad51abe32ad9099d5fe4085c473d54571f109d637a25d62885")
            .unwrap()
            .1
    );
}

#[test]
fn remove_data_request_runs_out_of_funds() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();

    test_info
        .remove_data_request(
            dr_id,
            vec![
                // burn all the funds
                DistributionMessage::Burn(DistributionBurn { amount: 20u128.into() }),
                // then try to reward the executor
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    amount:   10u128.into(),
                    identity: alice.pub_key_hex(),
                }),
            ],
        )
        .unwrap();
    assert_eq!(1, test_info.executor_balance("alice"));
}

#[test]
fn remove_data_request_with_more_drs_in_the_pool() {
    let mut test_info = TestInfo::init();

    // post 2 drs
    let mut alice = test_info.new_executor("alice", Some(42), Some(1));
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id1 = test_info
        .post_data_request(&mut alice, dr1, vec![], vec![], 1, None)
        .unwrap();
    let dr_id2 = test_info
        .post_data_request(&mut alice, dr2, vec![], vec![], 1, None)
        .unwrap();

    // Same commits & reveals for all drs
    let alice_reveal = RevealBody {
        id:                dr_id1.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };

    assert_eq!(
        2,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 100)
            .data_requests
            .len()
    );
    // Commit 2 drs
    test_info
        .commit_result(&alice, &dr_id1, alice_reveal.try_hash().unwrap())
        .unwrap();
    test_info
        .commit_result(&alice, &dr_id2, alice_reveal.try_hash().unwrap())
        .unwrap();
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 100)
            .data_requests
            .len()
    );
    assert_eq!(
        2,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 100)
            .data_requests
            .len()
    );

    // reveal first dr
    test_info.reveal_result(&alice, &dr_id1, alice_reveal.clone()).unwrap();
    assert_eq!(
        1,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 100)
            .data_requests
            .len()
    );

    // Check drs to be tallied
    let dr_to_be_tallied = test_info.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100);
    assert!(!dr_to_be_tallied.is_paused);
    assert_eq!(1, dr_to_be_tallied.data_requests.len());
    assert_eq!(dr_to_be_tallied.data_requests[0].id, dr_id1);

    // Remove only first dr ready to be tallied (while there is another one in the pool and not ready)
    // This checks part of the swap_remove logic
    let dr = dr_to_be_tallied.data_requests[0].clone();
    test_info
        .remove_data_request(
            dr.id,
            vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
                amount:   10u128.into(),
                identity: alice.pub_key_hex(),
            })],
        )
        .unwrap();
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100)
            .data_requests
            .len()
    );

    // Reveal the other dr
    test_info.reveal_result(&alice, &dr_id2, alice_reveal.clone()).unwrap();
    let dr_to_be_tallied = test_info.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100);
    assert!(!dr_to_be_tallied.is_paused);
    assert_eq!(1, dr_to_be_tallied.data_requests.len());

    // Remove last dr
    let dr = dr_to_be_tallied.data_requests[0].clone();
    test_info
        .remove_data_request(
            dr.id,
            vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
                amount:   10u128.into(),
                identity: alice.pub_key_hex(),
            })],
        )
        .unwrap();

    // Check dr to be tallied is empty
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100)
            .data_requests
            .len()
    );
}

#[test]
fn unstake_before_dr_removal_rewards_staker() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(22), None);

    let mut bob = test_info.new_executor("bob", Some(22), Some(1));

    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // bob commits a data result
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              bob.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&bob, &dr_id, bob_reveal.try_hash().unwrap())
        .unwrap();
    test_info.reveal_result(&bob, &dr_id, bob_reveal.clone()).unwrap();

    // bob unstakes before the data request is removed
    bob.unstake(&mut test_info, 1).unwrap();

    test_info
        .remove_data_request(
            dr_id,
            vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
                identity: bob.pub_key_hex(),
                amount:   5u128.into(),
            })],
        )
        .unwrap();

    // bob should still get the reward
    // get the staker info for the executor
    let staker = test_info.get_staker(bob.pub_key()).unwrap();
    assert_eq!(5, staker.tokens_pending_withdrawal.u128());

    // bob can withdraw the reward
    test_info.withdraw(&mut bob, 5).unwrap();
}
