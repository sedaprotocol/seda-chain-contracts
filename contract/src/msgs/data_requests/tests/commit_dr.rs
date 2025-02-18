use seda_common::{
    msgs::{data_requests::DataRequestStatus, staking::StakingConfig},
    types::HashSelf,
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_if_not_staked() {
    let test_info = TestInfo::init();
    let bob = test_info.new_account("bob", 22);

    // register an executor
    test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = bob.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    bob.commit_result(&dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"commit\")")]
fn cannot_commit_if_timed_out() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // set the block height to be equal to the timeout height
    test_info.set_block_height(11);

    // commit a data result
    alice.commit_result(&dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_on_expired_dr() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // set the block height to be later than the timeout
    test_info.set_block_height(11);
    // expire the data request
    test_info.creator().expire_data_requests().unwrap();

    // commit a data result
    anyone.commit_result(&dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn cannot_commit_if_not_enough_staked() {
    let test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake_to_register:               200u8.into(),
        minimum_stake_for_committee_eligibility: 100u8.into(),
        allowlist_enabled:                       false,
    };

    // owner sets staking config
    test_info.creator().set_staking_config(new_config).unwrap();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    anyone.commit_result(&dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
fn commit_result() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    test_info.new_executor("bob", 2, 1);
    test_info.new_executor("claire", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // check if executor can commit
    let query_result = alice.can_executor_commit(&dr_id, "0xcommitment".hash());
    assert!(query_result, "executor should be able to commit");

    // commit a data result
    alice.commit_result(&dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the committing state before meeting the replication factor
    let commiting = alice.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!commiting.is_paused);
    assert_eq!(1, commiting.data_requests.len());
    assert!(commiting.data_requests.iter().any(|r| r.id == dr_id));
}
#[test]
fn commits_meet_replication_factor() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    anyone.commit_result(&dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the revealing state after meeting the replication factor
    let revealing = anyone.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn cannot_double_commit() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    alice.commit_result(&dr_id, "0xcommitment1".hash()).unwrap();

    // check if executor can commit, should be false
    let query_result = alice.can_executor_commit(&dr_id, "0xcommitment2".hash());
    assert!(!query_result, "executor should not be able to commit");

    // try to commit again as the same user
    alice.commit_result(&dr_id, "0xcommitment2".hash()).unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn cannot_commit_after_replication_factor_reached() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    anyone.commit_result(&dr_id, "0xcommitment".hash()).unwrap();

    // commit again as a different user
    let new = test_info.new_executor("new", 2, 1);
    new.commit_result(&dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "verify: invalid proof")]
fn commits_wrong_signature_fails() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 9, None).unwrap();

    // commit a data result
    anyone
        .commit_result_wrong_height(&dr_id, "0xcommitment".hash())
        .unwrap();
}
