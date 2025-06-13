use std::collections::HashMap;

use seda_common::{msgs::data_requests::RevealBody, types::HashSelf};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
fn empty_works() {
    let test_info = TestInfo::init();
    let someone = test_info.new_executor("someone", 22, 1);

    let drs = someone.get_pending_data_requests(None, 10);
    assert!(!drs.is_paused);
    assert_eq!(0, drs.data_requests.len());
}

#[test]
fn one_works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone.post_data_request(dr, vec![], vec![], 2, None).unwrap();

    let drs = anyone.get_pending_data_requests(None, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());
    assert!(drs.data_requests.iter().any(|r| r.id == dr_id));
}

#[test]
fn limit_works() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 62, 1);

    // post multiple data requests
    for i in 0..3 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
    }

    let drs = alice.get_pending_data_requests(None, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn offset_works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 62, 1);

    // post multiple data requests
    for i in 0..3 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        anyone.post_data_request(dr, vec![], vec![], 1, None).unwrap();
    }

    let drs_one = anyone.get_pending_data_requests(None, 1);
    assert_eq!(1, drs_one.data_requests.len());
    assert_eq!(drs_one.last_seen_index.map(|(_, h, _)| h), Some(u64::MAX - 1));

    let drs = anyone.get_pending_data_requests(drs_one.last_seen_index, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn works_with_pagination_and_total() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 2 + 25 * 20, 1);

    let mut reveal_messages = HashMap::new();
    for i in 0..5 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
        let alice_reveal = RevealBody {
            dr_id:             dr_id.clone(),
            dr_block_height:   1,
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };
        reveal_messages.insert(dr_id, alice.create_reveal_message(alice_reveal));
    }

    // Initial state
    let first_page_response = alice.get_pending_data_requests(None, 2);
    assert_eq!(2, first_page_response.data_requests.len());

    let second_page_response = alice.get_pending_data_requests(first_page_response.last_seen_index, 2);
    assert_eq!(2, second_page_response.data_requests.len());

    let third_page_response = alice.get_pending_data_requests(second_page_response.last_seen_index, 2);
    assert_eq!(1, third_page_response.data_requests.len());

    // Commit one request and verify it's no longer in pending
    let dr_id_to_commit = &first_page_response.data_requests[1].id;
    let reveal_message = &reveal_messages[dr_id_to_commit];
    alice.commit_result(dr_id_to_commit, reveal_message).unwrap();

    // Get fresh first page after commit
    let new_first_page = alice.get_pending_data_requests(None, 2);
    assert_eq!(2, new_first_page.data_requests.len());

    // Get second page using new last_seen_index
    let new_second_page = alice.get_pending_data_requests(new_first_page.last_seen_index, 2);
    assert_eq!(2, new_second_page.data_requests.len());
}

#[test]
fn pause_works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let _dr_id = anyone.post_data_request(dr, vec![], vec![], 2, None).unwrap();

    let drs = anyone.get_pending_data_requests(None, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.creator().pause().unwrap();
    assert!(test_info.creator().is_paused());

    let drs = anyone.get_pending_data_requests(None, 10);
    assert!(drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.creator().unpause().unwrap();
    assert!(!test_info.creator().is_paused());
}
