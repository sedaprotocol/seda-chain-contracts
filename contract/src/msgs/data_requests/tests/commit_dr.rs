use seda_common::{
    msgs::{
        data_requests::{DataRequestStatus, RevealBody},
        staking::StakingConfig,
    },
    types::HashSelf,
};

use crate::{consts::INITIAL_COMMIT_TIMEOUT_IN_BLOCKS, msgs::data_requests::test_helpers, TestInfo};

#[test]
#[should_panic(expected = "not found")]
fn fails_if_not_staked() {
    let test_info = TestInfo::init();
    let bob = test_info.new_account("bob", 22);

    // register an executor
    test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = bob.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
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
}

#[test]
#[should_panic(expected = "DataRequestExpired(51, \"commit\")")]
fn fails_if_timed_out() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // set the block height to be equal to the timeout height
    test_info.set_block_height(INITIAL_COMMIT_TIMEOUT_IN_BLOCKS + 1);

    // commit a data result
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
}

#[test]
#[should_panic(expected = "not found")]
fn fails_on_expired_dr() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // set the block height to be later than the timeout
    test_info.set_block_height(INITIAL_COMMIT_TIMEOUT_IN_BLOCKS + 1);
    // expire the data request
    test_info.creator().expire_data_requests().unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn fails_if_not_enough_staked() {
    let test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake:     200u8.into(),
        allowlist_enabled: false,
    };

    // owner sets staking config
    test_info.creator().set_staking_config(new_config).unwrap();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();
}

#[test]
fn works() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    test_info.new_executor("bob", 2, 1);
    test_info.new_executor("claire", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);

    // check if executor can commit
    let query_result = alice.can_executor_commit(&dr_id, &alice_reveal_message);
    assert!(query_result, "executor should be able to commit");

    // commit a data result
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // check if the data request is in the committing state before meeting the
    // replication factor
    let commiting = alice.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert!(!commiting.is_paused);
    assert_eq!(1, commiting.data_requests.len());
    assert!(commiting.data_requests.iter().any(|r| r.base.id == dr_id));
}
#[test]
fn must_meet_replication_factor() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();

    // check if the data request is in the revealing state after meeting the
    // replication factor
    let revealing = anyone.get_data_requests_by_status(DataRequestStatus::Revealing, None, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn fails_double_commit() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
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

    // check if executor can commit, should be false
    let query_result = alice.can_executor_commit(&dr_id, &alice_reveal_message);
    assert!(!query_result, "executor should not be able to commit");

    // try to commit again as the same user
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn fails_after_reveal_started() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();

    // commit again as a different user
    let new = test_info.new_executor("new", 2, 1);
    let new_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let new_reveal_message = new.create_reveal_message(new_reveal);
    new.commit_result(&dr_id, &new_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "verify: invalid proof")]
fn wrong_signature_fails() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 9, None).unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone
        .commit_result_wrong_height(&dr_id, anyone_reveal_message)
        .unwrap();
}

#[test]
#[should_panic(expected = "NotOnAllowlist")]
fn must_be_on_allowlist_to_commit() {
    let test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // then we enable the allowlist
    let new_config = StakingConfig {
        minimum_stake:     1u128.into(),
        allowlist_enabled: true,
    };

    // owner sets staking config
    test_info.creator().set_staking_config(new_config).unwrap();

    // commit a data result
    let anyone_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let anyone_reveal_message = anyone.create_reveal_message(anyone_reveal);
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();
}
