use seda_common::{
    msgs::data_requests::{DataRequestStatus, DrConfig, RevealBody},
    types::HashSelf,
};

use crate::TestInfo;

#[test]
pub fn returns_pause_property_dr_query_by_status() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 22, 1);

    // post a data request
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let _dr_id = anyone.post_data_request(dr, vec![], vec![], 2, None).unwrap();

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.creator().pause().unwrap();
    assert!(test_info.creator().is_paused());

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert!(drs.is_paused);
    assert_eq!(1, drs.data_requests.len());

    test_info.creator().unpause().unwrap();
    assert!(!test_info.creator().is_paused());
}

#[test]
pub fn execute_messages_get_paused() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 1000, 1);

    // post a data request we can commit on
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id_committable = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // post a data request we can reveal on
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id_revealable = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
    // commit on it
    let alice_reveal = RevealBody {
        dr_id:             dr_id_revealable.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id_revealable, &alice_reveal_message).unwrap();

    // pause the contract
    test_info.creator().pause().unwrap();
    assert!(test_info.creator().is_paused());

    // try to post another data request
    let dr = crate::msgs::data_requests::test_helpers::calculate_dr_id_and_args(3, 1);
    let res = alice.post_data_request(dr, vec![], vec![], 1, None);
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // try to commit a data result
    let res = alice.commit_result(&dr_id_committable, &alice_reveal_message);
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // try to reveal a data result
    let res = alice.reveal_result(alice_reveal_message);
    assert!(res.is_err_and(|e| e.to_string().contains("pause")));

    // can still change the timeout config
    let dr_config = DrConfig {
        commit_timeout_in_blocks:        1.try_into().unwrap(),
        reveal_timeout_in_blocks:        1.try_into().unwrap(),
        backup_delay_in_blocks:          1.try_into().unwrap(),
        dr_reveal_size_limit_in_bytes:   1024.try_into().unwrap(),
        exec_input_limit_in_bytes:       2048.try_into().unwrap(),
        tally_input_limit_in_bytes:      512.try_into().unwrap(),
        consensus_filter_limit_in_bytes: 512.try_into().unwrap(),
        memo_limit_in_bytes:             512.try_into().unwrap(),
    };
    test_info.creator().set_dr_config(dr_config).unwrap();
}
