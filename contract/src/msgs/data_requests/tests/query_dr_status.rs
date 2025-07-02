use seda_common::{
    msgs::data_requests::{
        sudo::{DistributionExecutorReward, DistributionMessage},
        DataRequestStatus,
        RevealBody,
    },
    types::HashSelf,
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
fn empty_works() {
    let test_info = TestInfo::init();
    let someone = test_info.new_executor("someone", 22, 1);

    let drs = someone.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
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

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());
    assert!(drs.data_requests.iter().any(|r| r.base.id == dr_id));
}

#[test]
fn query_drs_statuses_work() {
    let test_info = TestInfo::init();
    // create an executor
    let alice = test_info.new_executor("alice", 62, 1);
    // create another executor
    let bob = test_info.new_executor("bob", 2, 1);

    // post several data requests
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);

    let dr_id1 = alice.post_data_request(dr1, vec![], vec![], 1, None).unwrap();
    let dr_id2 = alice.post_data_request(dr2, vec![], vec![], 2, None).unwrap();
    let dr_id3 = alice.post_data_request(dr3, vec![], vec![], 3, None).unwrap();

    // query the statuses of the data requests and assert all are in committing
    // status
    let dr_ids = vec![dr_id1.to_string(), dr_id2.to_string(), dr_id3.to_string()];
    let statuses = alice.get_data_requests_statuses(dr_ids.clone());
    assert_eq!(3, statuses.len());
    for dr in statuses {
        assert_eq!(Some(DataRequestStatus::Committing), dr.1);
    }

    // upgrade one dr to revealing
    let alice_reveal = RevealBody {
        dr_id:             dr_id1.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let alice_reveal_message = alice.create_reveal_message(alice_reveal);
    alice.commit_result(&dr_id1, &alice_reveal_message).unwrap();

    let bob_reveal = RevealBody {
        dr_id:             dr_id1.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    let bob_reveal_message = bob.create_reveal_message(bob_reveal);
    bob.commit_result(&dr_id1, &bob_reveal_message).unwrap();

    // upgrade the second dr to tallying
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
    alice.reveal_result(alice_reveal_message).unwrap();

    // query the statuses of the data requests again and assert that one is in each
    // status
    let statuses = alice.get_data_requests_statuses(dr_ids.clone());
    assert_eq!(3, statuses.len());
    assert_eq!(Some(DataRequestStatus::Revealing), statuses[&dr_id1]);
    assert_eq!(Some(DataRequestStatus::Tallying), statuses[&dr_id2]);
    assert_eq!(Some(DataRequestStatus::Committing), statuses[&dr_id3]);

    // sudo resolve the tally dr
    test_info
        .creator()
        .remove_data_request(
            dr_id2.clone(),
            vec![DistributionMessage::ExecutorReward(DistributionExecutorReward {
                amount:   10u128.into(),
                identity: alice.pub_key_hex(),
            })],
        )
        .unwrap();

    // query the statuses of the data requests again and assert that the second dr
    // is now none
    let statuses = alice.get_data_requests_statuses(dr_ids);
    assert_eq!(3, statuses.len());
    assert_eq!(Some(DataRequestStatus::Revealing), statuses[&dr_id1]);
    assert_eq!(None, statuses[&dr_id2]);
    assert_eq!(Some(DataRequestStatus::Committing), statuses[&dr_id3]);
}

#[test]
fn limit_works() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 62, 1);
    test_info.new_executor("bob", 2, 1);
    test_info.new_executor("claire", 2, 1);

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 3);
    alice.post_data_request(dr1, vec![], vec![], 1, None).unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 3);
    alice.post_data_request(dr2, vec![], vec![], 2, None).unwrap();

    // post a third data request
    let dr3: seda_common::msgs::data_requests::PostDataRequestArgs = test_helpers::calculate_dr_id_and_args(3, 3);
    alice.post_data_request(dr3, vec![], vec![], 3, None).unwrap();

    let drs = alice.get_data_requests_by_status(DataRequestStatus::Committing, None, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn sort_by_posted_gas_price_works() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 62, 1);

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr1_gas_price = 2001;
    let dr1_funds: u128 = dr1_gas_price * (dr1.exec_gas_limit + dr1.tally_gas_limit) as u128;
    alice
        .post_data_request(dr1, vec![], vec![], 1, Some(dr1_funds))
        .unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    let dr2_gas_price = 2003;
    let dr2_funds: u128 = dr2_gas_price * (dr2.exec_gas_limit + dr2.tally_gas_limit) as u128;
    alice
        .post_data_request(dr2, vec![], vec![], 2, Some(dr2_funds))
        .unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);
    let dr3_gas_price = 2002;
    let dr3_funds: u128 = dr3_gas_price * (dr3.exec_gas_limit + dr3.tally_gas_limit) as u128;
    alice
        .post_data_request(dr3, vec![], vec![], 3, Some(dr3_funds))
        .unwrap();

    let drs = alice.get_data_requests_by_status(DataRequestStatus::Committing, None, 3);
    println!(
        "drs: {:?}",
        drs.data_requests
            .iter()
            .map(|dr| dr.base.posted_gas_price.u128())
            .collect::<Vec<_>>()
    );
    assert_eq!(3, drs.data_requests.len());
    assert_eq!(dr2_gas_price, drs.data_requests[0].base.posted_gas_price.u128());
    assert_eq!(dr3_gas_price, drs.data_requests[1].base.posted_gas_price.u128());
    assert_eq!(dr1_gas_price, drs.data_requests[2].base.posted_gas_price.u128());
}

#[test]
fn offset_works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 62, 1);

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    let posted_dr1 = anyone.post_data_request(dr1, vec![], vec![], 1, None).unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    anyone.post_data_request(dr2, vec![], vec![], 2, None).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);
    anyone.post_data_request(dr3.clone(), vec![], vec![], 3, None).unwrap();

    let drs_one = anyone.get_data_requests_by_status(DataRequestStatus::Committing, None, 1);
    assert_eq!(1, drs_one.data_requests.len());
    assert!(drs_one.data_requests.iter().any(|dr| dr.base.id == posted_dr1));
    assert_eq!(drs_one.last_seen_index.as_ref().map(|(_, h, _)| h.as_str()), Some("1"));

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, drs_one.last_seen_index, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn works_with_more_drs_in_pool() {
    let test_info = TestInfo::init();

    let alice = test_info.new_executor("alice", 2 + 25 * 20, 1);

    for i in 0..25 {
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
        let alice_reveal_message = alice.create_reveal_message(alice_reveal);

        if i < 15 {
            alice.commit_result(&dr_id, &alice_reveal_message).unwrap();
        }

        if i < 3 {
            alice.reveal_result(alice_reveal_message).unwrap();
        }
    }

    assert_eq!(
        10,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 10)
            .data_requests
            .len()
    );
    assert_eq!(
        12,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 15)
            .data_requests
            .len()
    );
    assert_eq!(
        3,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 15)
            .data_requests
            .len()
    );
}

#[test]
fn works_with_many_more_drs_in_pool() {
    let test_info = TestInfo::init();

    // This test posts 163 data requests
    let alice = test_info.new_executor("alice", 2 + 163 * 20, 1);

    for i in 0..100 {
        let dr = test_helpers::calculate_dr_id_and_args(i, 1);
        let dr_id = alice.post_data_request(dr.clone(), vec![], vec![], 1, None).unwrap();
        let alice_reveal = RevealBody {
            dr_id:             dr_id.clone(),
            dr_block_height:   1,
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };
        let alice_reveal_message = alice.create_reveal_message(alice_reveal);

        if i % 2 == 0 {
            alice.commit_result(&dr_id, &alice_reveal_message).unwrap();

            alice.get_data_requests_by_status(DataRequestStatus::Committing, None, 100);

            let dr = test_helpers::calculate_dr_id_and_args(i + 20000, 1);
            alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
        }
    }
    assert_eq!(
        100,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        50,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        0,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 1000)
            .data_requests
            .len()
    );

    for (i, request) in alice
        .get_data_requests_by_status(DataRequestStatus::Revealing, None, 1000)
        .data_requests
        .into_iter()
        .enumerate()
    {
        if i % 4 == 0 {
            let alice_reveal = RevealBody {
                dr_id:             request.base.id.clone(),
                dr_block_height:   1,
                reveal:            "10".hash().into(),
                gas_used:          0,
                exit_code:         0,
                proxy_public_keys: vec![],
            };
            let alice_reveal_message = alice.create_reveal_message(alice_reveal);

            alice.reveal_result(alice_reveal_message).unwrap();

            let dr = test_helpers::calculate_dr_id_and_args(i as u128 + 10000, 1);
            alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
        }
    }

    assert_eq!(
        113,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        13,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 1000)
            .data_requests
            .len()
    );

    for (i, request) in alice
        .get_data_requests_by_status(DataRequestStatus::Tallying, None, 1000)
        .data_requests
        .into_iter()
        .enumerate()
    {
        if i % 8 == 0 {
            alice
                .remove_data_request(
                    request.base.id.to_string(),
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
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, None, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        11,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, None, 1000)
            .data_requests
            .len()
    );
}
