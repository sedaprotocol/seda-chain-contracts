use seda_common::{
    msgs::data_requests::{DataRequestStatus, RevealBody},
    types::{HashSelf, ToHexStr, TryHashSelf},
};

use crate::{msgs::data_requests::test_helpers, new_public_key, TestInfo};

#[test]
fn reveal_result() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    let query_result = bob.can_executor_reveal(&dr_id);
    assert!(
        !query_result,
        "executor should not be able to reveal before DR is in the revealing state"
    );

    // bob also commits
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    bob.commit_result(&dr_id, bob_reveal.try_hash().unwrap()).unwrap();

    let query_result = alice.can_executor_reveal(&dr_id);
    assert!(query_result, "executor should be able to reveal");
    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn reveal_result_with_proxies() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    let (_, proxy1) = new_public_key();
    let (_, proxy2) = new_public_key();
    let proxies = vec![proxy1.to_hex(), proxy2.to_hex()];

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: proxies,
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();

    let tallying = alice.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 10);
    assert!(!tallying.is_paused);
    assert_eq!(1, tallying.data_requests.len());
    assert!(tallying.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "InvalidHexCharacter")]
fn reveal_result_with_proxies_not_valid_public_keys() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    let proxy1 = "proxy1".to_string();
    let proxy2 = "proxy2".to_string();
    let proxies = vec![proxy1, proxy2];

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: proxies.clone(),
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_result_reveal_body_missing_proxies() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    let (_, proxy1) = new_public_key();
    let (_, proxy2) = new_public_key();
    let proxies = vec![proxy1.to_hex(), proxy2.to_hex()];

    // alice commits a data result
    let mut alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: proxies,
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // alice reveals
    alice_reveal.proxy_public_keys = vec![];
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealNotStarted")]
fn cannot_reveal_if_commit_rf_not_met() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    let query_result = bob.can_executor_reveal(&dr_id);
    assert!(
        !query_result,
        "executor should not be able to reveal if they did not commit"
    );

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"reveal\")")]
fn cannot_reveal_if_timed_out() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // set the block height to be later than the timeout
    test_info.set_block_height(11);

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_reveal_on_expired_dr() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // set the block height to be later than the timeout
    test_info.set_block_height(11);

    // expire the data request
    test_info.creator().expire_data_requests().unwrap();

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn cannot_reveal_if_user_did_not_commit() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", 22, 1);
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", 2, 1);
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };

    // bob reveals
    bob.reveal_result(&dr_id, bob_reveal).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn cannot_double_reveal() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // bob also commits
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    bob.commit_result(&dr_id, bob_reveal.try_hash().unwrap()).unwrap();

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal.clone()).unwrap();

    // alice reveals again
    alice.reveal_result(&dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_must_match_commitment() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice
        .commit_result(
            &dr_id,
            RevealBody {
                id:                dr_id.clone(),
                salt:              alice.salt(),
                reveal:            "11".hash().into(),
                gas_used:          0,
                exit_code:         0,
                proxy_public_keys: vec![],
            }
            .try_hash()
            .unwrap(),
        )
        .unwrap();

    // bob also commits

    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    bob.commit_result(&dr_id, bob_reveal.try_hash().unwrap()).unwrap();

    // alice reveals
    alice.reveal_result(&dr_id, alice_reveal).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn can_reveal_after_unstaking() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

    // alice unstakes after committing
    alice.unstake(1).unwrap();

    // alice should still be able to reveal
    alice.reveal_result(&dr_id, alice_reveal.clone()).unwrap();

    // verify the request moved to tallying state
    let tallying = alice.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 10);
    assert!(!tallying.is_paused);
    assert_eq!(1, tallying.data_requests.len());
    assert!(tallying.data_requests.iter().any(|r| r.id == dr_id));
}
