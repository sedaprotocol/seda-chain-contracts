use seda_common::{
    msgs::data_requests::{DataRequestStatus, RevealBody, TimeoutConfig},
    types::{HashSelf, TryHashSelf},
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

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

    let alice = test_info.new_executor("alice", Some(2), None);
    test_info.set_timeout_config(&alice, timeout_config).unwrap();
}

#[test]
fn timed_out_requests_move_to_tally() {
    let mut test_info = TestInfo::init();
    let mut alice = test_info.new_executor("alice", Some(42), Some(1));

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
