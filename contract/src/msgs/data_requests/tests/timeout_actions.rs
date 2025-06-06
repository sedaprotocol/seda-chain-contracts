use seda_common::{
    msgs::data_requests::{DataRequestStatus, DrConfig, RevealBody},
    types::HashSelf,
};

use crate::{consts::INITIAL_DR_CONFIG, msgs::data_requests::test_helpers, TestInfo};

#[test]
fn owner_can_update_dr_config() {
    let test_info = TestInfo::init();

    let dr_config = DrConfig {
        commit_timeout_in_blocks:        1.try_into().unwrap(),
        reveal_timeout_in_blocks:        1.try_into().unwrap(),
        backup_delay_in_blocks:          1.try_into().unwrap(),
        dr_reveal_size_limit_in_bytes:   1.try_into().unwrap(),
        exec_input_limit_in_bytes:       1.try_into().unwrap(),
        tally_input_limit_in_bytes:      1.try_into().unwrap(),
        consensus_filter_limit_in_bytes: 1.try_into().unwrap(),
        memo_limit_in_bytes:             1.try_into().unwrap(),
        payback_address_limit_in_bytes:  1.try_into().unwrap(),
        seda_payload_limit_in_bytes:     1.try_into().unwrap(),
    };

    test_info.creator().set_dr_config(dr_config).unwrap();
}

#[test]
#[should_panic(expected = "NotOwner")]
fn only_owner_can_change_dr_config() {
    let test_info = TestInfo::init();

    let dr_config = DrConfig {
        commit_timeout_in_blocks:        1.try_into().unwrap(),
        reveal_timeout_in_blocks:        1.try_into().unwrap(),
        backup_delay_in_blocks:          1.try_into().unwrap(),
        dr_reveal_size_limit_in_bytes:   1.try_into().unwrap(),
        exec_input_limit_in_bytes:       1.try_into().unwrap(),
        tally_input_limit_in_bytes:      1.try_into().unwrap(),
        consensus_filter_limit_in_bytes: 1.try_into().unwrap(),
        memo_limit_in_bytes:             1.try_into().unwrap(),
        payback_address_limit_in_bytes:  1.try_into().unwrap(),
        seda_payload_limit_in_bytes:     1.try_into().unwrap(),
    };

    let alice = test_info.new_account("alice", 2);
    alice.set_dr_config(dr_config).unwrap();
}

#[test]
fn timed_out_requests_move_to_tally() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 42, 1);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();

    // set the block height to the height it would timeout
    test_info.set_block_height(INITIAL_DR_CONFIG.commit_timeout_in_blocks.get() as u64 + 1);

    // process the timed out requests at current height
    test_info.creator().expire_data_requests().unwrap();

    // post another data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr_id2 = alice
        .post_data_request(
            dr2,
            vec![],
            vec![],
            INITIAL_DR_CONFIG.commit_timeout_in_blocks.get() as u64 + 1,
            None,
        )
        .unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        dr_id:             dr_id2.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id2, &alice_reveal_message).unwrap();

    // set the block height to be later than the timeout so it times out during the
    // reveal phase
    test_info.set_block_height(
        INITIAL_DR_CONFIG
            .commit_timeout_in_blocks
            .saturating_add(INITIAL_DR_CONFIG.reveal_timeout_in_blocks.get())
            .get() as u64
            + 1,
    );

    // process the timed out requests at current height
    test_info.creator().expire_data_requests().unwrap();

    // check that the request is now in the tallying state
    let tallying = alice
        .get_data_requests_by_status(DataRequestStatus::Tallying, None, 10)
        .data_requests
        .into_iter()
        .map(|r| r.base.id)
        .collect::<Vec<_>>();
    assert_eq!(2, tallying.len());
    assert_eq!(tallying[0], dr_id);
    assert_eq!(tallying[1], dr_id2);
}
