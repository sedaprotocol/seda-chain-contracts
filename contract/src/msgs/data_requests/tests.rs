use super::*;
use crate::{new_public_key, TestInfo};

#[test]
fn query_drs_by_status_has_none() {
    let test_info = TestInfo::init();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(0, drs.len());
}

#[test]
fn query_drs_by_status_has_one() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 2).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, drs.len());
    assert!(drs.iter().any(|r| r.id == dr_id));
}

#[test]
fn query_drs_by_status_limit_works() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let mut claire = test_info.new_executor("claire", Some(2));
    claire.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 3);
    test_info.post_data_request(&alice, dr1, vec![], vec![], 1).unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 3);
    test_info.post_data_request(&alice, dr2, vec![], vec![], 2).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 3);
    test_info.post_data_request(&alice, dr3, vec![], vec![], 3).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 2);
    assert_eq!(2, drs.len());
}

#[test]
fn query_drs_by_status_offset_works() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    test_info.post_data_request(&anyone, dr1, vec![], vec![], 1).unwrap();

    // post a scond data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    test_info.post_data_request(&anyone, dr2, vec![], vec![], 2).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);
    test_info.post_data_request(&anyone, dr3, vec![], vec![], 3).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 1, 2);
    assert_eq!(2, drs.len());
}

#[test]
fn post_data_request() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();

    // data request with id 0x69... does not yet exist
    let value = test_info.get_data_request("69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f");
    assert_eq!(None, value);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&anyone, dr.clone(), vec![], vec![1, 2, 3], 1)
        .unwrap();

    // expect an error when trying to post it again
    let res = test_info.post_data_request(&anyone, dr.clone(), vec![], vec![1, 2, 3], 1);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = test_info.get_data_request(&dr_id);
    assert_eq!(Some(test_helpers::construct_dr(dr, vec![], 1)), received_value);
    let await_commits = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, await_commits.len());
    assert!(await_commits.iter().any(|r| r.id == dr_id));

    // nonexistent data request does not yet exist
    let value = test_info.get_data_request("00f0f00f0f00f0f0000000f0fff0ff0ff0ffff0fff00000f000ff000000f000f");
    assert_eq!(None, value);
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_if_not_staked() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let bob = test_info.new_executor("bob", Some(2));

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&bob, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&bob, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"commit\")")]
fn cannot_commit_if_timed_out() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
fn commit_result() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let mut claire = test_info.new_executor("claire", Some(2));
    claire.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&alice, &dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the committing state before meeting the replication factor
    let commiting = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, commiting.len());
    assert!(commiting.iter().any(|r| r.id == dr_id));
}
#[test]
fn commits_meet_replication_factor() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the revealing state after meeting the replication factor
    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealing.len());
    assert!(revealing.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn cannot_double_commit() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&alice, &dr_id, "0xcommitment1".hash()).unwrap();

    // try to commit again as the same user
    test_info.commit_result(&alice, &dr_id, "0xcommitment2".hash()).unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn cannot_commit_after_replication_factor_reached() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, &dr_id, "0xcommitment".hash()).unwrap();

    // commit again as a different user
    let mut new = test_info.new_executor("new", Some(2));
    new.stake(&mut test_info, 1).unwrap();
    test_info.commit_result(&new, &dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "verify: invalid proof")]
fn commits_wrong_signature_fails() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut anyone = test_info.new_executor("anyone", Some(2));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 9).unwrap();

    // commit a data result
    test_info
        .commit_result_wrong_height(&anyone, &dr_id, "0xcommitment".hash())
        .unwrap();
}

#[test]
fn reveal_result() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // bob also commits
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&bob, &dr_id, bob_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();

    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealing.len());
    assert!(revealing.iter().any(|r| r.id == dr_id));
}

#[test]
fn reveal_result_with_proxies() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();

    let tallying = test_info.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 10);
    assert_eq!(1, tallying.len());
    assert!(tallying.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "InvalidHexCharacter")]
fn reveal_result_with_proxies_not_valid_public_keys() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_result_reveal_body_missing_proxies() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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
    test_info
        .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    alice_reveal.proxy_public_keys = vec![];
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealNotStarted")]
fn cannot_reveal_if_commit_rf_not_met() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"reveal\")")]
fn cannot_reveal_if_timed_out() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // set the block height to be later than the timeout
    test_info.set_block_height(11);

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_reveal_on_expired_dr() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // set the block height to be later than the timeout
    test_info.set_block_height(11);

    // expire the data request
    test_info.expire_data_requests().unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn cannot_reveal_if_user_did_not_commit() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // bob also commits
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };

    // bob reveals
    test_info.reveal_result(&bob, &dr_id, bob_reveal).unwrap();

    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealing.len());
    assert!(revealing.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn cannot_double_reveal() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // bob also commits
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&bob, &dr_id, bob_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();

    // alice reveals again
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_must_match_commitment() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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
        .commit_result(
            &alice,
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
    test_info
        .commit_result(&bob, &dr_id, bob_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();

    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealing.len());
    assert!(revealing.iter().any(|r| r.id == dr_id));
}

#[test]
fn post_data_result() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // owner posts a data result
    let dr = test_info.get_data_request(&dr_id).unwrap();
    let result = test_helpers::construct_result(dr, alice_reveal, 0);
    test_info.post_data_result(dr_id.clone(), result, 0).unwrap();

    // check we can get the results
    let _res1 = test_info.get_data_result(&dr_id);
}

#[test]
fn post_data_results() {
    let mut test_info = TestInfo::init();

    // post data request 1
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id1 = test_info.post_data_request(&alice, dr1, vec![], vec![], 1).unwrap();

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
    let dr_id2 = test_info.post_data_request(&alice, dr2, vec![], vec![], 2).unwrap();

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
    let dr1 = test_info.get_data_request(&dr_id1).unwrap();
    let result1 = test_helpers::construct_result(dr1, alice_reveal1, 0);
    let dr2 = test_info.get_data_request(&dr_id2).unwrap();
    let result2 = test_helpers::construct_result(dr2, alice_reveal2, 0);
    test_info
        .post_data_results(vec![(dr_id1.clone(), result1, 0), (dr_id2.clone(), result2, 0)])
        .unwrap();

    // check we can get the results
    let _res1 = test_info.get_data_result(&dr_id1);
    let _res2 = test_info.get_data_result(&dr_id2);
}

#[test]
#[should_panic = "NotEnoughReveals"]
fn cant_post_if_replication_factor_not_met() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

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

    // bob also commits
    let bob_reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              alice.salt(),
        reveal:            "20".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&bob, &dr_id, bob_reveal.try_hash().unwrap())
        .unwrap();

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();

    // post a data result
    let dr = test_info.get_data_request(&dr_id).unwrap();
    let result = test_helpers::construct_result(dr, alice_reveal, 0);
    test_info.post_data_result(dr_id, result, 0).unwrap();
}

#[test]
fn check_data_request_id() {
    // Expected DR ID for following DR:
    // {
    //     "version": "0.0.1",
    //     "exec_program_id": "044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
    //     "dr_inputs": "ZHJfaW5wdXRz",
    //     "tally_program_id": "3a1561a3d854e446801b339c137f87dbd2238f481449c00d3470cfcc2a4e24a1",
    //     "tally_inputs": "dGFsbHlfaW5wdXRz",
    //     "replication_factor": 1,
    //     "consensus_filter": "AA==",
    //     "gas_price": 10,
    //     "gas_limit": 10,
    //     "memo": "XTtTqpLgvyGr54/+ov83JyG852lp7VqzBrC10UpsIjg="
    //   }
    let expected_dr_id = "2399c8b5ea3cea169496647e3ec54edbad9e920c567b650272115afb89d6b6b1";

    // compute and check if dr id matches expected value
    let dr = test_helpers::calculate_dr_id_and_args(0, 1);
    let dr_id = dr.try_hash().unwrap();
    assert_eq!(hex::encode(dr_id), expected_dr_id);
}

#[test]
fn check_data_result_id() {
    // Expected RESULT ID for the following Data Result:
    // {
    //     "version": "0.0.1",
    //     "dr_id": "74d7e8c9a77b7b4777153a32fcdf2424489f24cd59d3043eb2a30be7bba48306",
    //     "consensus": true,
    //     "exit_code": 0,
    //     "result": "Ghkvq84TmIuEmU1ClubNxBjVXi8df5QhiNQEC5T8V6w=",
    //     "block_height": 12345,
    //     "gas_used": 20,
    //     "payback_address": "",
    //     "seda_payload": ""
    //   }
    let expected_result_id = "74d7e8c9a77b7b4777153a32fcdf2424489f24cd59d3043eb2a30be7bba48306";
    let dr_args = test_helpers::calculate_dr_id_and_args(0, 1);

    // reveal sample
    let alice_reveal = RevealBody {
        id:                expected_result_id.to_owned(),
        salt:              "123".into(),
        reveal:            "10".hash().into(),
        gas_used:          20,
        exit_code:         0,
        proxy_public_keys: vec![],
    };

    // check if data result id matches expected value
    let dr = test_helpers::construct_dr(dr_args, vec![0x04, 0x05, 0x06], 12345);
    let result = test_helpers::construct_result(dr, alice_reveal, 0);

    let result_id = result.try_hash().unwrap();

    assert_eq!(hex::encode(result_id), expected_result_id);
}

#[test]
fn post_data_result_with_more_drs_in_the_pool() {
    let mut test_info = TestInfo::init();

    // post 2 drs
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id1 = test_info.post_data_request(&alice, dr1, vec![], vec![], 1).unwrap();
    let dr_id2 = test_info.post_data_request(&alice, dr2, vec![], vec![], 1).unwrap();

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
            .len()
    );
    assert_eq!(
        2,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 100)
            .len()
    );

    // reveal first dr
    test_info.reveal_result(&alice, &dr_id1, alice_reveal.clone()).unwrap();
    assert_eq!(
        1,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 100)
            .len()
    );

    // Check drs to be tallied
    let dr_to_be_tallied = test_info.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100);
    assert_eq!(1, dr_to_be_tallied.len());
    assert_eq!(dr_to_be_tallied[0].id, dr_id1);

    // Post only first dr ready to be tallied (while there is another one in the pool and not ready)
    // This checks part of the swap_remove logic
    let dr = dr_to_be_tallied[0].clone();
    let result1 = test_helpers::construct_result(dr.clone(), alice_reveal.clone(), 0);
    test_info.post_data_result(dr.id, result1, 0).unwrap();
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100)
            .len()
    );

    // Reveal the other dr
    test_info.reveal_result(&alice, &dr_id2, alice_reveal.clone()).unwrap();
    let dr_to_be_tallied = test_info.get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100);
    assert_eq!(1, dr_to_be_tallied.len());

    // Post last dr result
    let dr = dr_to_be_tallied[0].clone();
    let result1 = test_helpers::construct_result(dr.clone(), alice_reveal, 0);
    test_info.post_data_result(dr.id, result1, 0).unwrap();

    // Check dr to be tallied is empty
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 100)
            .len()
    );
}

#[test]
fn get_data_requests_by_status_with_more_drs_in_pool() {
    let mut test_info = TestInfo::init();

    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    for i in 0..25 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();
        let alice_reveal = RevealBody {
            id:                dr_id.clone(),
            salt:              alice.salt(),
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };

        if i < 15 {
            test_info
                .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
                .unwrap();
        }

        if i < 3 {
            test_info.reveal_result(&alice, &dr_id, alice_reveal.clone()).unwrap();
        }
    }

    assert_eq!(
        10,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 10)
            .len()
    );
    assert_eq!(
        12,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 15)
            .len()
    );
    assert_eq!(
        3,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 15)
            .len()
    );
}

#[test]
fn get_data_requests_by_status_with_many_more_drs_in_pool() {
    let mut test_info = TestInfo::init();

    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    for i in 0..100 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = test_info
            .post_data_request(&alice, dr.clone(), vec![], vec![], 1)
            .unwrap();
        let alice_reveal = RevealBody {
            id:                dr_id.clone(),
            salt:              alice.salt(),
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };

        if i % 2 == 0 {
            test_info
                .commit_result(&alice, &dr_id, alice_reveal.try_hash().unwrap())
                .unwrap();

            // test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 100);

            let dr = test_helpers::calculate_dr_id_and_args(i + 20000, 1);
            test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();
        }
    }
    assert_eq!(
        100,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .len()
    );
    assert_eq!(
        50,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .len()
    );
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .len()
    );

    for (i, request) in test_info
        .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
        .into_iter()
        .enumerate()
    {
        if i % 4 == 0 {
            let alice_reveal = RevealBody {
                id:                request.id.clone(),
                salt:              alice.salt(),
                reveal:            "10".hash().into(),
                gas_used:          0,
                exit_code:         0,
                proxy_public_keys: vec![],
            };

            test_info
                .reveal_result(&alice, &request.id, alice_reveal.clone())
                .unwrap();

            let dr = test_helpers::calculate_dr_id_and_args(i as u128 + 10000, 1);
            test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();
        }
    }

    assert_eq!(
        113,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .len()
    );
    assert_eq!(
        37,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .len()
    );
    assert_eq!(
        13,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .len()
    );

    for (i, request) in test_info
        .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
        .into_iter()
        .enumerate()
    {
        if i % 8 == 0 {
            let alice_reveal = RevealBody {
                id:                request.id.clone(),
                salt:              alice.salt(),
                reveal:            "10".hash().into(),
                gas_used:          0,
                exit_code:         0,
                proxy_public_keys: vec![],
            };
            let dr_info = test_info.get_data_request(&request.id).unwrap();
            let result = test_helpers::construct_result(dr_info.clone(), alice_reveal.clone(), 0);
            test_info.post_data_result(request.id.to_string(), result, 0).unwrap();
        }
    }
    assert_eq!(
        113,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .len()
    );
    assert_eq!(
        37,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .len()
    );
    assert_eq!(
        11,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .len()
    );
}

#[test]
fn post_data_request_replication_factor_too_high() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    let sender = test_info.new_executor("sender", Some(2));

    // post a data request with rf=1
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let res = test_info.post_data_request(&sender, dr.clone(), vec![], vec![1, 2, 3], 1);
    assert!(res.is_ok());

    // post a data request with rf=2
    // expect an error when trying to post it again
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let res = test_info.post_data_request(&sender, dr.clone(), vec![], vec![1, 2, 3], 1);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestReplicationFactorTooHigh(1)));
}

#[test]
#[should_panic(expected = "DataRequestReplicationFactorZero")]
fn post_data_request_replication_factor_zero() {
    let mut test_info = TestInfo::init();
    let sender = test_info.new_executor("sender", Some(2));

    // post a data request with rf=0
    let dr = test_helpers::calculate_dr_id_and_args(1, 0);
    test_info
        .post_data_request(&sender, dr.clone(), vec![], vec![1, 2, 3], 1)
        .unwrap();
}

#[test]
fn timed_out_requests_move_to_tally() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // set the block height to the height it would timeout
    test_info.set_block_height(11);

    // process the timed out requests at current height
    test_info.expire_data_requests().unwrap();

    // post another data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id2 = test_info.post_data_request(&alice, dr2, vec![], vec![], 11).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        id:                dr_id2.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id2, alice_reveal.try_hash().unwrap())
        .unwrap();

    // set the block height to be later than the timeout so it times out during the reveal phase
    test_info.set_block_height(21);

    // process the timed out requests at current height
    test_info.expire_data_requests().unwrap();

    // check that the request is now in the tallying state
    let tallying = test_info
        .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 10)
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    assert_eq!(2, tallying.len());
    assert_eq!(tallying[0], dr_id);
    assert_eq!(tallying[1], dr_id2);
}

#[test]
fn owner_can_update_timeout_config() {
    let mut test_info = TestInfo::init();

    let timeout_config = TimeoutConfig {
        commit_timeout_in_blocks: 1,
        reveal_timeout_in_blocks: 1,
    };

    test_info
        .set_timeout_config(&test_info.creator(), timeout_config)
        .unwrap();
}

#[test]
#[should_panic(expected = "NotOwner")]
fn only_owner_can_change_timeout_config() {
    let mut test_info = TestInfo::init();

    let timeout_config = TimeoutConfig {
        commit_timeout_in_blocks: 1,
        reveal_timeout_in_blocks: 1,
    };

    let alice = test_info.new_executor("alice", Some(2));
    test_info.set_timeout_config(&alice, timeout_config).unwrap();
}
