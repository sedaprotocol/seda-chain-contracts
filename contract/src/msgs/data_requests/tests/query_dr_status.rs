use seda_common::{
    msgs::data_requests::{
        sudo::{DistributionExecutorReward, DistributionMessage},
        DataRequestStatus,
        RevealBody,
    },
    types::{HashSelf, TryHashSelf},
};

use crate::{msgs::data_requests::test_helpers, TestInfo};

#[test]
fn empty_works() {
    let test_info = TestInfo::init();
    let someone = test_info.new_executor("someone", 22, 1);

    let drs = someone.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
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

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!drs.is_paused);
    assert_eq!(1, drs.data_requests.len());
    assert!(drs.data_requests.iter().any(|r| r.id == dr_id));
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
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 3);
    alice.post_data_request(dr3, vec![], vec![], 3, None).unwrap();

    let drs = alice.get_data_requests_by_status(DataRequestStatus::Committing, 0, 2);
    assert!(!drs.is_paused);
    assert_eq!(2, drs.data_requests.len());
}

#[test]
fn offset_works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 62, 1);

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 1);
    anyone.post_data_request(dr1, vec![], vec![], 1, None).unwrap();

    // post a scond data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 1);
    anyone.post_data_request(dr2, vec![], vec![], 2, None).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 1);
    anyone.post_data_request(dr3, vec![], vec![], 3, None).unwrap();

    let drs = anyone.get_data_requests_by_status(DataRequestStatus::Committing, 1, 2);
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
            id:                dr_id.clone(),
            salt:              alice.salt(),
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };

        if i < 15 {
            alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();
        }

        if i < 3 {
            alice.reveal_result(&dr_id, alice_reveal.clone()).unwrap();
        }
    }

    assert_eq!(
        10,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 10)
            .data_requests
            .len()
    );
    assert_eq!(
        12,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 15)
            .data_requests
            .len()
    );
    assert_eq!(
        3,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 15)
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
            id:                dr_id.clone(),
            salt:              alice.salt(),
            reveal:            "10".hash().into(),
            gas_used:          0,
            exit_code:         0,
            proxy_public_keys: vec![],
        };

        if i % 2 == 0 {
            alice.commit_result(&dr_id, alice_reveal.try_hash().unwrap()).unwrap();

            alice.get_data_requests_by_status(DataRequestStatus::Committing, 0, 100);

            let dr = test_helpers::calculate_dr_id_and_args(i + 20000, 1);
            alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
        }
    }
    assert_eq!(
        100,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        50,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        0,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );

    for (i, request) in alice
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

            alice.reveal_result(&request.id, alice_reveal.clone()).unwrap();

            let dr = test_helpers::calculate_dr_id_and_args(i as u128 + 10000, 1);
            alice.post_data_request(dr, vec![], vec![], 1, None).unwrap();
        }
    }

    assert_eq!(
        113,
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        13,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );

    for (i, request) in alice
        .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
        .data_requests
        .into_iter()
        .enumerate()
    {
        if i % 8 == 0 {
            alice
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
        alice
            .get_data_requests_by_status(DataRequestStatus::Committing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        37,
        alice
            .get_data_requests_by_status(DataRequestStatus::Revealing, 0, 1000)
            .data_requests
            .len()
    );
    assert_eq!(
        11,
        alice
            .get_data_requests_by_status(DataRequestStatus::Tallying, 0, 1000)
            .data_requests
            .len()
    );
}
