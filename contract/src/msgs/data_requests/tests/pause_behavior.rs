use seda_common::{
    msgs::data_requests::{DataRequestStatus, RevealBody, TimeoutConfig},
    types::{HashSelf, TryHashSelf},
};

use crate::TestInfo;

#[test]
pub fn paused_contract_returns_pause_property_dr_query_by_status() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
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
    let mut alice = test_info.new_executor("alice", 1000, 1);

    // post a data request we can commit on
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id_committable = test_info
        .post_data_request(&mut alice, dr, vec![], vec![], 1, None)
        .unwrap();

    // post a data request we can reveal on
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(2, 1);
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
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(3, 1);
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
