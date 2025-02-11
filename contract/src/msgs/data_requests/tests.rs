use std::collections::HashMap;

use msgs::data_requests::sudo::{
    DistributionBurn,
    DistributionDataProxyReward,
    DistributionExecutorReward,
    DistributionMessage,
};
use state::DR_ESCROW;

use super::*;
use crate::{new_public_key, TestInfo};

#[test]
fn query_drs_by_status_has_none() {
    let test_info = TestInfo::init();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!drs.is_paused);
    assert_eq!(0, drs.data_requests.len());
}

#[test]
fn query_drs_by_status_has_one() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 2, None)
        .unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());
    assert!(drs.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn query_drs_by_status_limit_works() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(62));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let mut claire = test_info.new_executor("claire", Some(2));
    claire.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 3);
    test_info
        .post_data_request(&mut alice, dr1, vec![], vec![], 1, None)
        .unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 3);
    test_info
        .post_data_request(&mut alice, dr2, vec![], vec![], 2, None)
        .unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 3);
    test_info
        .post_data_request(&mut alice, dr3, vec![], vec![], 3, None)
        .unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn query_drs_by_status_offset_works() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(62));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    test_info
        .post_data_request(&mut anyone, dr1, vec![], vec![], 1, None)
        .unwrap();

    // post a scond data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    test_info
        .post_data_request(&mut anyone, dr2, vec![], vec![], 2, None)
        .unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);
    test_info
        .post_data_request(&mut anyone, dr3, vec![], vec![], 3, None)
        .unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 1, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn post_data_request() {
    let mut test_info = TestInfo::init();
    // let c = test_info.creator().addr();

    let mut anyone = test_info.new_executor("anyone", Some(52));
    anyone.stake(&mut test_info, 1).unwrap();

    // data request... does not yet exist
    let value = test_info.get_data_request("673842e9aaa751cb7430630a8706b6d8e6280f3ab8d06cb45c44d57738988236");
    assert_eq!(None, value);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // Expect the dr staked to exist and be correct
    let staked = DR_ESCROW
        .load(
            &*test_info.app().contract_storage(&test_info.contract_addr()),
            &Hash::from_hex_str(&dr_id).unwrap(),
        )
        .unwrap();
    assert_eq!(20, staked.amount.u128());
    assert_eq!(anyone.addr(), staked.poster);

    // expect an error when trying to post it again
    let res = test_info.post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = test_info.get_data_request(&dr_id);
    assert_eq!(Some(test_helpers::construct_dr(dr, vec![], 1)), received_value);
    let await_commits = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!await_commits.is_paused);
    assert_eq!(1, await_commits.data_requests.len());
    assert!(await_commits.data_requests.iter().any(|r| r.id == dr_id));

    // nonexistent data request does not yet exist
    let value = test_info.get_data_request("00f0f00f0f00f0f0000000f0fff0ff0ff0ffff0fff00000f000ff000000f000f");
    assert_eq!(None, value);
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn post_dr_with_not_enough_funds_fails() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.exec_gas_limit = 1000;
    test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 2, None)
        .unwrap();
}

#[test]
fn post_dr_with_max_gas_limits() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(u128::MAX));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    // set gas price to 1 to make the gas limit calculation easier
    dr.gas_price = Uint128::from(1u128);
    dr.exec_gas_limit = u64::MAX;
    dr.tally_gas_limit = u64::MAX;
    test_info
        .post_data_request(
            &mut anyone,
            dr,
            vec![],
            vec![],
            2,
            Some(u128::from(u64::MAX) + u128::from(u64::MAX)),
        )
        .unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_commit_if_not_staked() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(22));

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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();
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
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();
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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();
    let mut claire = test_info.new_executor("claire", Some(2));
    claire.stake(&mut test_info, 1).unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();
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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 1, None)
        .unwrap();

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
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 9, None)
        .unwrap();

    // commit a data result
    test_info
        .commit_result_wrong_height(&anyone, &dr_id, "0xcommitment".hash())
        .unwrap();
}

#[test]
fn reveal_result() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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

    let query_result = test_info.can_executor_reveal(&dr_id, &bob.pub_key_hex());
    assert!(
        !query_result,
        "executor should not be able to reveal before DR is in the revealing state"
    );

    // bob also commits
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

    let query_result = test_info.can_executor_reveal(&dr_id, &alice.pub_key_hex());
    assert!(query_result, "executor should be able to reveal");
    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();

    let revealing = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn reveal_result_with_proxies() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

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
    assert!(!tallying.is_paused);
    assert_eq!(1, tallying.data_requests.len());
    assert!(tallying.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "InvalidHexCharacter")]
fn reveal_result_with_proxies_not_valid_public_keys() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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

    let query_result = test_info.can_executor_reveal(&dr_id, &bob.pub_key_hex());
    assert!(
        !query_result,
        "executor should not be able to reveal if they did not commit"
    );

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "DataRequestExpired(11, \"reveal\")")]
fn cannot_reveal_if_timed_out() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
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

    // set the block height to be later than the timeout
    test_info.set_block_height(11);

    // alice reveals
    test_info.reveal_result(&alice, &dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "not found")]
fn cannot_reveal_on_expired_dr() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
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
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn cannot_double_reveal() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let mut bob = test_info.new_executor("bob", Some(2));
    bob.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
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
    assert!(!revealing.is_paused);
    assert_eq!(1, revealing.data_requests.len());
    assert!(revealing.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn remove_data_request() {
    let mut test_info = TestInfo::init();

    // post a data request
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();
    let mut executor = test_info.new_executor("exec", Some(51));
    executor.stake(&mut test_info, 1).unwrap();

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
    let bob = test_info.new_executor("bob", Some(2));
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
    let mut alice = test_info.new_executor("alice", Some(22));
    alice.stake(&mut test_info, 1).unwrap();
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();
    let mut executor = test_info.new_executor("exec", Some(51));
    executor.stake(&mut test_info, 1).unwrap();

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
    let mut alice = test_info.new_executor("alice", Some(42));
    alice.stake(&mut test_info, 1).unwrap();
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
fn check_data_request_id() {
    // Expected DR ID for following DR:
    // {
    //     "version": "0.0.1",
    //     "exec_program_id": "044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
    //     "exec_inputs": "ZHJfaW5wdXRz",
    //     "exec_gas_limit": 1,
    //     "tally_program_id": "3a1561a3d854e446801b339c137f87dbd2238f481449c00d3470cfcc2a4e24a1",
    //     "tally_inputs": "dGFsbHlfaW5wdXRz",
    //     "tally_gas_limit": 1,
    //     "replication_factor": 1,
    //     "consensus_filter": "AA==",
    //     "gas_price": 10,
    //     "memo": "XTtTqpLgvyGr54/+ov83JyG852lp7VqzBrC10UpsIjg="
    //   }
    let expected_dr_id = "2404059f879876ad51abe32ad9099d5fe4085c473d54571f109d637a25d62885";

    // compute and check if dr id matches expected value
    let dr = test_helpers::calculate_dr_id_and_args(0, 1);
    let dr_id = dr.try_hash().unwrap();
    assert_eq!(hex::encode(dr_id), expected_dr_id);
}

#[test]
fn remove_data_request_with_more_drs_in_the_pool() {
    let mut test_info = TestInfo::init();

    // post 2 drs
    let mut alice = test_info.new_executor("alice", Some(42));
    alice.stake(&mut test_info, 1).unwrap();
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
fn get_data_requests_by_status_with_more_drs_in_pool() {
    let mut test_info = TestInfo::init();

    let mut alice = test_info.new_executor("alice", Some(2 + 25 * 20));
    alice.stake(&mut test_info, 1).unwrap();

    for i in 0..25 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = test_info
            .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
            .unwrap();
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
            .data_requests
            .len()
    );
    assert_eq!(
        12,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 15)
            .data_requests
            .len()
    );
    assert_eq!(
        3,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 15)
            .data_requests
            .len()
    );
}

#[test]
fn get_data_requests_by_status_with_many_more_drs_in_pool() {
    let mut test_info = TestInfo::init();

    // This test posts 163 data requests
    let mut alice = test_info.new_executor("alice", Some(2 + 163 * 20));
    alice.stake(&mut test_info, 1).unwrap();

    for i in 0..100 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = test_info
            .post_data_request(&mut alice, dr.clone(), vec![], vec![], 1, None)
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
            test_info
                .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
                .unwrap();
        }
    }
    assert_eq!(
        100,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        50,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        0,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );

    for (i, request) in test_info
        .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
        .data_requests
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
            test_info
                .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
                .unwrap();
        }
    }

    assert_eq!(
        113,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        13,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );

    for (i, request) in test_info
        .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
        .data_requests
        .into_iter()
        .enumerate()
    {
        if i % 8 == 0 {
            test_info
                .remove_data_request(
                    request.id.to_string(),
                    vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
                        amount:   10u128.into(),
                        identity: alice.pub_key_hex(),
                    })],
                )
                .unwrap();
        }
    }
    assert_eq!(
        113,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        11,
        test_info
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );
}

#[test]
fn post_data_request_replication_factor_too_high() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(2));
    alice.stake(&mut test_info, 1).unwrap();

    let mut sender = test_info.new_executor("sender", Some(42));

    // post a data request with rf=1
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let res = test_info.post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_ok());

    // post a data request with rf=2
    // expect an error when trying to post it again
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let res = test_info.post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestReplicationFactorTooHigh(1)));
}

#[test]
#[should_panic(expected = "DataRequestReplicationFactorZero")]
fn post_data_request_replication_factor_zero() {
    let mut test_info = TestInfo::init();
    let mut sender = test_info.new_executor("sender", Some(22));

    // post a data request with rf=0
    let dr = test_helpers::calculate_dr_id_and_args(1, 0);
    test_info
        .post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();
}

#[test]
fn timed_out_requests_move_to_tally() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(42));
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // set the block height to the height it would timeout
    test_info.set_block_height(11);

    // process the timed out requests at current height
    test_info.expire_data_requests().unwrap();

    // post another data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id2 = test_info
        .post_data_request(&mut alice, dr2, vec![], vec![], 11, None)
        .unwrap();

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
        .data_requests
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

#[test]
pub fn paused_contract_returns_pause_property_dr_query_by_status() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(22));
    anyone.stake(&mut test_info, 1).unwrap();

    // post a data request
    let dr = crate::msgs::data_requests::test::test_helpers::calculate_dr_id_and_args(1, 1);
    let _dr_id = test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 2, None)
        .unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.pause(&test_info.creator()).unwrap();
    assert!(test_info.is_paused());

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.unpause(&test_info.creator()).unwrap();
    assert!(!test_info.is_paused());
}

#[test]
pub fn execute_messages_get_paused() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(1000));

    // register alice as a staker
    alice.stake(&mut test_info, 1).unwrap();

    // post a data request we can commit on
    let dr = crate::msgs::data_requests::test::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id_committable = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // post a data request we can reveal on
    let dr = crate::msgs::data_requests::test::test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id_revealable = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();
    // commit on it
    let alice_reveal = RevealBody {
        id:                dr_id_revealable.clone(),
        salt:              alice.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    test_info
        .commit_result(&alice, &dr_id_revealable, alice_reveal.try_hash().unwrap())
        .unwrap();

    // pause the contract
    test_info.pause(&test_info.creator()).unwrap();
    assert!(test_info.is_paused());

    // try to post another data request
    let dr = crate::msgs::data_requests::test::test_helpers::calculate_dr_id_and_args(3, 1);
    let res = test_info.post_data_request(&mut alice, dr, vec![], vec![], 1, None);
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // try to commit a data result
    let res = test_info.commit_result(&alice, &dr_id_committable, alice_reveal.try_hash().unwrap());
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // try to reveal a data result
    let res = test_info.reveal_result(&alice, &dr_id_revealable, alice_reveal.clone());
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // can still change the timeout config
    let timeout_config = TimeoutConfig {
        commit_timeout_in_blocks: 1,
        reveal_timeout_in_blocks: 1,
    };
    test_info
        .set_timeout_config(&test_info.creator(), timeout_config)
        .unwrap();
}
