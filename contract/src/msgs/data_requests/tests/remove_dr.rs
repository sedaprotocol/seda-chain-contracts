use std::collections::HashMap;

use cosmwasm_std::Uint128;
use seda_common::{
    msgs::data_requests::{
        sudo::{DistributionBurn, DistributionDataProxyReward, DistributionExecutorReward, DistributionMessage},
        DataRequestStatus,
        RevealBody,
    },
    types::{HashSelf, ToHexStr},
};

use crate::{
    msgs::data_requests::{consts::min_post_dr_cost, test_helpers},
    new_public_key,
    seda_to_aseda,
    TestInfo,
};

#[test]
fn basic_workflow_works() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_account("alice", 22);
    let executor = test_info.new_executor("exec", 51, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let executor_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let executor_reveal_message = executor.create_reveal_message(executor_reveal);
    executor.commit_result(&dr_id, &executor_reveal_message).unwrap();
    executor.reveal_result(executor_reveal_message).unwrap();

    // owner removes a data result
    // data proxy reward goes to executors SEDA address
    // reward goes to executors identity
    // invalid identities and address are burned
    // non staked executor is not rewarded
    // remainder refunds to alice
    let (_, proxy) = new_public_key();
    let bob = test_info.new_account("bob", 2);
    test_info
        .creator()
        .remove_data_request(
            dr_id,
            vec![
                DistributionMessage::Burn(DistributionBurn { amount: 1u128.into() }),
                // valid data proxy reward
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    payout_address: executor.addr().to_string(),
                    amount:         5u128.into(),
                    public_key:     proxy.to_hex(),
                }),
                // valid executor reward
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: executor.pub_key_hex(),
                    amount:   5u128.into(),
                }),
                // invalid data proxy reward
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    amount:         2u128.into(),
                    payout_address: "invalid".to_string(),
                    public_key:     proxy.to_hex(),
                }),
                // invalid executor reward
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: "invalid".to_string(),
                    amount:   2u128.into(),
                }),
                // valid executor reward
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    identity: bob.pub_key_hex(),
                    amount:   2u128.into(),
                }),
            ],
        )
        .unwrap();
    // Alice seda - stake amount minus the rewards
    let alice_expected_balance = seda_to_aseda(22.into()) - 1 - 5 - 5 - 2 - 2 - 2;
    assert_eq!(alice_expected_balance, test_info.executor_balance("alice"));
    // Bob seda - should have had no changes
    let bob_expected_balance = seda_to_aseda(2.into());
    assert_eq!(bob_expected_balance, test_info.executor_balance("bob"));
    // Executor seda - data proxy reward minus stake amount
    let executor_expected_balance = seda_to_aseda(51.into()) + 5 - 1;
    assert_eq!(executor_expected_balance, test_info.executor_balance("exec"));

    // get the staker info for the executor
    // should have 5 seda pending withdrawal from executor reward
    let staker = executor.get_staker_info().unwrap();
    assert_eq!(5, staker.tokens_pending_withdrawal.u128());
}

#[test]
fn retains_order() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_account("alice", 1);
    let executor = test_info.new_executor("exec", 1, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // the executor commits a data result
    let executor_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let executor_reveal_message = executor.create_reveal_message(executor_reveal);
    executor.commit_result(&dr_id, &executor_reveal_message).unwrap();
    executor.reveal_result(executor_reveal_message).unwrap();

    let (_, proxy) = new_public_key();

    // owner removes a data result
    // reward goes to executor
    // remainder refunds to alice
    test_info
        .creator()
        .remove_data_request(
            dr_id,
            vec![
                DistributionMessage::Burn(DistributionBurn { amount: 10u128.into() }),
                DistributionMessage::Burn(DistributionBurn { amount: 8u128.into() }),
                DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                    payout_address: executor.addr().to_string(),
                    amount:         3u128.into(),
                    public_key:     proxy.to_hex(),
                }),
            ],
        )
        .unwrap();
    // Alice seda - balance minus the rewards
    let alice_expected_balance = seda_to_aseda(1.into()) - 10 - 8 - 3;
    assert_eq!(alice_expected_balance, test_info.executor_balance("alice"));
    // Executor seda - balance minus the stake amount + the data proxy reward
    let executor_expected_balance = seda_to_aseda(1.into()) - 1 + 3;
    assert_eq!(executor_expected_balance, test_info.executor_balance("exec"));
}

#[test]
fn status_codes_work() {
    let test_info = TestInfo::init();

    // post data request 1
    let alice = test_info.new_executor("alice", 1, 1);
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id1 = alice.post_data_request(dr1, vec![], vec![], 1, None).unwrap();

    // alice commits data result 1
    let alice_reveal1 = RevealBody {
        dr_id:             dr_id1.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal1_message = alice.create_reveal_message(alice_reveal1);
    alice.commit_result(&dr_id1, &alice_reveal1_message).unwrap();
    alice.reveal_result(alice_reveal1_message).unwrap();

    // post data request 2
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id2 = alice.post_data_request(dr2, vec![], vec![], 2, None).unwrap();

    // alice commits data result 2
    let alice_reveal2 = RevealBody {
        dr_id:             dr_id2.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal2_message = alice.create_reveal_message(alice_reveal2);
    alice.commit_result(&dr_id2, &alice_reveal2_message).unwrap();
    alice.reveal_result(alice_reveal2_message).unwrap();

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
    let removed = test_info.creator().remove_data_requests(to_remove).unwrap();
    removed.iter().for_each(|r| assert_eq!(0, r.1));
}

#[test]
fn invalid_status_codes_work() {
    let test_info = TestInfo::init();

    // remove a dr with an invalid dr_id and dr that does not exist
    let mut to_remove = HashMap::new();
    to_remove.insert("does_not_exist".to_string(), vec![]);
    to_remove.insert(
        "2404059f879876ad51abe32ad9099d5fe4085c473d54571f109d637a25d62885".to_string(),
        vec![],
    );
    let removed = test_info.creator().remove_data_requests(to_remove).unwrap();
    // test this way since tests are not in wasm so hashmap order is non
    // deterministic
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
fn works_when_runs_out_of_funds() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", 1, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();
    alice.reveal_result(alice_reveal_message).unwrap();

    test_info
        .creator()
        .remove_data_request(
            dr_id,
            vec![
                // burn all the funds
                DistributionMessage::Burn(DistributionBurn {
                    amount: Uint128::new(min_post_dr_cost()),
                }),
                // then try to reward the executor
                DistributionMessage::ExecutorReward(DistributionExecutorReward {
                    amount:   10u128.into(),
                    identity: alice.pub_key_hex(),
                }),
            ],
        )
        .unwrap();
    // Alice seda - balance minus the rewards minus the stake amount
    let alice_expected_balance = seda_to_aseda(1.into()) - min_post_dr_cost() - 1;
    assert_eq!(alice_expected_balance, test_info.executor_balance("alice"));
}

#[test]
fn works_with_more_drs_in_the_pool() {
    let test_info = TestInfo::init();

    // post 2 drs
    let alice = test_info.new_executor("alice", 42, 1);
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id1 = alice.post_data_request(dr1, vec![], vec![], 1, None).unwrap();
    let dr_id2 = alice.post_data_request(dr2, vec![], vec![], 1, None).unwrap();

    let alice_reveal1 = RevealBody {
        dr_id:             dr_id1.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal1_message = alice.create_reveal_message(alice_reveal1);
    let alice_reveal2 = RevealBody {
        dr_id:             dr_id2.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal2_message = alice.create_reveal_message(alice_reveal2);
    assert_eq!(
        2,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 100)
            .data_requests
            .len()
    );
    // Commit 2 drs
    alice.commit_result(&dr_id1, &alice_reveal1_message).unwrap();
    alice.commit_result(&dr_id2, &alice_reveal2_message).unwrap();
    assert_eq!(
        0,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 100)
            .data_requests
            .len()
    );
    assert_eq!(
        2,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 100)
            .data_requests
            .len()
    );

    // reveal first dr
    alice.reveal_result(alice_reveal1_message).unwrap();
    assert_eq!(
        1,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 100)
            .data_requests
            .len()
    );

    // Check drs to be tallied
    let dr_to_be_tallied = alice.get_data_requests_by_status(DataRequestStatus::Tallying, None, 100);
    assert!(!dr_to_be_tallied.is_paused);
    assert_eq!(1, dr_to_be_tallied.data_requests.len());
    assert_eq!(dr_to_be_tallied.data_requests[0].id, dr_id1);

    // Remove only first dr ready to be tallied (while there is another one in the
    // pool and not ready) This checks part of the swap_remove logic
    let dr = dr_to_be_tallied.data_requests[0].clone();
    alice
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
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 100)
            .data_requests
            .len()
    );

    // Reveal the other dr
    alice.reveal_result(alice_reveal2_message).unwrap();
    let dr_to_be_tallied = alice.get_data_requests_by_status(DataRequestStatus::Tallying, None, 100);
    assert!(!dr_to_be_tallied.is_paused);
    assert_eq!(1, dr_to_be_tallied.data_requests.len());

    // Remove last dr
    let dr = dr_to_be_tallied.data_requests[0].clone();
    alice
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
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 100)
            .data_requests
            .len()
    );
}

#[test]
fn unstake_before_dr_removal_still_rewards_staker() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_account("alice", 22);

    let bob = test_info.new_executor("bob", 22, 1);

    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // bob commits a data result
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id, &bob_reveal_message).unwrap();
    bob.reveal_result(bob_reveal_message).unwrap();

    // bob unstakes before the data request is removed
    bob.unstake().unwrap();

    test_info
        .creator()
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
    let staker = bob.get_staker_info().unwrap();
    assert_eq!(5, staker.tokens_pending_withdrawal.u128());

    // bob can withdraw the reward
    bob.withdraw().unwrap();
}
