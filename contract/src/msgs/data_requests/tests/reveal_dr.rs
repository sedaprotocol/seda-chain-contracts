use seda_common::{
    msgs::data_requests::{DataRequestStatus, RevealBody},
    types::{HashSelf, ToHexStr},
};

use crate::{
    consts::INITIAL_DR_REVEAL_SIZE_LIMIT,
    error::ContractError,
    msgs::data_requests::test_helpers,
    new_public_key,
    TestInfo,
};

#[test]
fn works() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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

    let query_result = bob.can_executor_reveal(&dr_id);
    assert!(
        !query_result,
        "executor should not be able to reveal before DR is in the revealing state"
    );

    // bob also commits
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id, &bob_reveal_message).unwrap();

    let query_result = alice.can_executor_reveal(&dr_id);
    assert!(query_result, "executor should be able to reveal");
    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, None, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
fn works_with_proxies() {
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
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: proxies,
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();

    let tallying = alice.get_data_requests_by_status(DataRequestStatus::Tallying, None, 10);
    assert!(!tallying.is_paused);
    assert_eq!(1, tallying.data_requests.len());
    assert!(tallying.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
#[should_panic(expected = "InvalidHexCharacter")]
fn fails_with_invalid_proxies_public_keys() {
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
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: proxies.clone(),
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "RevealNotStarted")]
fn fails_if_not_in_reveal_status() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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

    let query_result = bob.can_executor_reveal(&dr_id);
    assert!(
        !query_result,
        "executor should not be able to reveal if they did not commit"
    );

    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(6, \"reveal\")")]
fn fails_if_timed_out() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
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

    // set the block height to be later than the timeout
    test_info.set_block_height(6);

    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn fails_on_expired_dr() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
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

    // set the block height to be later than the timeout
    test_info.set_block_height(6);

    // expire the data request
    test_info.creator().expire_data_requests().unwrap();

    // alice reveals
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn fails_if_user_did_not_commit() {
    let test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", 22, 1);
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

    // bob also commits
    let bob = test_info.new_executor("bob", 2, 1);
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);

    // bob reveals
    bob.reveal_result(bob_reveal_message).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, None, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn fails_on_double_reveal() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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

    // bob also commits
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id, &bob_reveal_message).unwrap();

    // alice reveals
    alice.reveal_result(alice_reveal_message.clone()).unwrap();

    // alice reveals again
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn fails_if_does_not_match_commitment() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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
    let alice_reveal_message_commitment = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message_commitment).unwrap();

    // bob also commits
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id, &bob_reveal_message).unwrap();

    // alice reveals a different message
    let alice_reveal2 = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "30".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal2);
    alice.reveal_result(alice_reveal_message).unwrap();

    let revealing = alice.get_data_requests_by_status(DataRequestStatus::Revealing, None, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn fails_if_proxy_public_keys_changed() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result
    let (_, proxy1) = new_public_key();
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![proxy1.to_hex()],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // alice reveals with an additional proxy public key
    let (_, proxy2) = new_public_key();
    let alice_reveal2 = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![proxy1.to_hex(), proxy2.to_hex()],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal2);
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
fn works_after_unstaking() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
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

    // alice unstakes after committing
    alice.unstake().unwrap();

    // alice should still be able to reveal
    alice.reveal_result(alice_reveal_message).unwrap();

    // verify the request moved to tallying state
    let tallying = alice.get_data_requests_by_status(DataRequestStatus::Tallying, None, 10);
    assert!(!tallying.is_paused);
    assert_eq!(1, tallying.data_requests.len());
    assert!(tallying.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn cannot_front_run_commit_reveal() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let fred = test_info.new_executor("fred", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice submits a commitment but fred front-runs it
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    fred.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // fred tries to front-run by copying alice's reveal message
    // this should fail since the reveal message is for alice's commitment
    fred.reveal_result(alice_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn cannot_front_run_reveal() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let fred = test_info.new_executor("fred", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice submits a commitment but fred front-runs it
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal.clone());
    fred.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // fred tries to copy the reveal body from alice and create their own reveal
    // message this should fail since fred's commitment was for alice's reveal
    // message
    let fred_reveal_message = fred.create_reveal_message(alice_reveal);
    fred.reveal_result(fred_reveal_message).unwrap();
}

#[test]
#[should_panic(expected = "RevealTooBig")]
fn reveal_too_big_rf1() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result with a reveal that is too big
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            [0; INITIAL_DR_REVEAL_SIZE_LIMIT + 1].into(), // too big for the DR
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // alice tries to reveal
    alice.reveal_result(alice_reveal_message).unwrap();
}

#[test]
fn reveal_too_big_rf2() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 22, 1);
    let bob = test_info.new_executor("bob", 2, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // alice commits a data result with a good sized reveal
    let alice_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            [0; 10].into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

    // bob commits a data result with a reveal that is too big
    let bob_reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            [0; (INITIAL_DR_REVEAL_SIZE_LIMIT / 2) + 1].into(), // too big for the DR
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id, &bob_reveal_message).unwrap();

    // alice tries to reveal
    alice.reveal_result(alice_reveal_message).unwrap();

    // bob tries to reveal
    let res = bob.reveal_result(bob_reveal_message);
    assert!(
        res.is_err_and(|x| x == ContractError::RevealTooBig),
        "Bob should not be able to reveal with a too big reveal"
    );
}
