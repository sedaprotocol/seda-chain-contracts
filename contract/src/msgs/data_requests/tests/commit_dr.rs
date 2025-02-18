use seda_common::{
    msgs::{data_requests::DataRequestStatus, staking::StakingConfig},
    types::HashSelf,
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_if_not_staked() {
    let mut test_info = TestInfo::init();
    let mut bob = test_info.new_executor("bob", Some(22), None);

    // register an executor
    test_info.new_executor("anyone", Some(22), Some(1));

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut bob, dr, vec![], vec![], 1, None)
        .unwrap();

    // commit a data result
    test_info.commit_result(&bob, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"commit\")")]
fn cannot_commit_if_timed_out() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // set the block height to be equal to the timeout height
    test_info.set_block_height(11);

    // commit a data result
    test_info.commit_result(&alice, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_on_expired_dr() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 1, None)
        .unwrap();

    // set the block height to be later than the timeout
    test_info.set_block_height(11);
    // expire the data request
    test_info.expire_data_requests().unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn cannot_commit_if_not_enough_staked() {
    let mut test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake_to_register:               200u8.into(),
        minimum_stake_for_committee_eligibility: 100u8.into(),
        allowlist_enabled:                       false,
    };

    // owner sets staking config
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 1, None)
        .unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
fn commit_result() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));
    test_info.new_executor("bob", Some(2), Some(1));
    test_info.new_executor("claire", Some(2), Some(1));

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // check if executor can commit
    let query_result = test_info.can_executor_commit(&alice, &dr_id, "0xcommitment".hash());
    assert!(query_result, "executor should be able to commit");

    // commit a data result
    test_info.commit_result(&alice, &dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the committing state before meeting the replication factor
    let commiting = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!commiting.is_paused);
    assert_eq!(1, commiting.data_requests.len());
    assert!(commiting.data_requests.iter().any(|r| r.id == dr_id));
}
#[test]
fn commits_meet_replication_factor() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 1, None)
        .unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the revealing state after meeting the replication factor
    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn cannot_double_commit() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22), Some(1));
    test_info.new_executor("bob", Some(2), Some(1));

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // commit a data result
    test_info.commit_result(&alice, &dr_id, "0xcommitment1".hash()).unwrap();

    // check if executor can commit, should be false
    let query_result = test_info.can_executor_commit(&alice, &dr_id, "0xcommitment2".hash());
    assert!(!query_result, "executor should not be able to commit");

    // try to commit again as the same user
    test_info.commit_result(&alice, &dr_id, "0xcommitment2".hash()).unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn cannot_commit_after_replication_factor_reached() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 1, None)
        .unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();

    // commit again as a different user
    let new = test_info.new_executor("new", Some(2), Some(1));
    test_info.commit_result(&new, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "verify: invalid proof")]
fn commits_wrong_signature_fails() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 9, None)
        .unwrap();

    // commit a data result
    test_info
        .commit_result_wrong_height(&anyone, &dr_id, "0xcommitment".hash())
        .unwrap();
}
