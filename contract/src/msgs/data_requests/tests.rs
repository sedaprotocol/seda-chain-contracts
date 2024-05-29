use super::*;
use crate::TestInfo;

#[test]
fn query_drs_by_status_has_none() {
    let test_info = TestInfo::init();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(0, drs.len());
}

#[test]
fn query_drs_by_status_has_one() {
    let mut test_info = TestInfo::init();

    let anyone = test_info.new_executor("anyone", Some(2));
    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 2).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, drs.len());
    assert!(drs.contains_key(&dr_id.to_hex()));
}

#[test]
fn query_drs_by_status_limit_works() {
    let mut test_info = TestInfo::init();

    let anyone = test_info.new_executor("anyone", Some(2));

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 3);
    test_info.post_data_request(&anyone, dr1, vec![], vec![], 1).unwrap();

    // post a second data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 3);
    test_info.post_data_request(&anyone, dr2, vec![], vec![], 2).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 3);
    test_info.post_data_request(&anyone, dr3, vec![], vec![], 3).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 2);
    assert_eq!(2, drs.len());
}

#[test]
fn query_drs_by_status_offset_works() {
    let mut test_info = TestInfo::init();

    let anyone = test_info.new_executor("anyone", Some(2));

    // post a data request
    let dr1 = test_helpers::calculate_dr_id_and_args(1, 3);
    test_info.post_data_request(&anyone, dr1, vec![], vec![], 1).unwrap();

    // post a scond data request
    let dr2 = test_helpers::calculate_dr_id_and_args(2, 3);
    test_info.post_data_request(&anyone, dr2, vec![], vec![], 2).unwrap();

    // post a third data request
    let dr3 = test_helpers::calculate_dr_id_and_args(3, 3);
    test_info.post_data_request(&anyone, dr3, vec![], vec![], 3).unwrap();

    let drs = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 1, 2);
    assert_eq!(2, drs.len());
}

#[test]
fn post_data_request() {
    let mut test_info = TestInfo::init();

    // data request with id 0x69... does not yet exist
    let value = test_info.get_dr("0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".hash());
    assert_eq!(None, value);

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info
        .post_data_request(&anyone, dr.clone(), vec![], vec![], 1)
        .unwrap();

    // expect an error when trying to post it again
    let res = test_info.post_data_request(&anyone, dr.clone(), vec![], vec![], 1);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = test_info.get_dr(dr_id);
    assert_eq!(Some(test_helpers::construct_dr(dr_id, dr, vec![], 1)), received_value);
    let await_commits = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, await_commits.len());
    assert!(await_commits.contains_key(&dr_id.to_hex()));

    // nonexistent data request does not yet exist
    let value = test_info.get_dr("nonexistent".hash());
    assert_eq!(None, value);
}

#[test]
fn commit_result() {
    let mut test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 3);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the committing state before meeting the replication factor
    let commiting = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(1, commiting.len());
    assert!(commiting.contains_key(&dr_id.to_hex()));
}

#[test]
fn commits_meet_replication_factor() {
    let mut test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, dr_id, "0xcommitment".hash()).unwrap();

    // check if the data request is in the revealing state after meeting the replication factor
    let commiting = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, commiting.len());
    assert!(commiting.contains_key(&dr_id.to_hex()));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn cannot_double_commit() {
    let mut test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, dr_id, "0xcommitment1".hash()).unwrap();

    // try to commit again as the same user
    test_info.commit_result(&anyone, dr_id, "0xcommitment2".hash()).unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn cannot_commit_after_replication_factor_reached() {
    let mut test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 1).unwrap();

    // commit a data result
    test_info.commit_result(&anyone, dr_id, "0xcommitment".hash()).unwrap();

    // commit again as a different user
    let new = test_info.new_executor("new", Some(2));
    test_info.commit_result(&new, dr_id, "0xcommitment".hash()).unwrap();
}

#[test]
#[should_panic(expected = "verify: invalid proof")]
fn commits_wrong_signature_fails() {
    let mut test_info = TestInfo::init();

    // post a data request
    let anyone = test_info.new_executor("anyone", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&anyone, dr, vec![], vec![], 9).unwrap();

    // commit a data result
    test_info
        .commit_result_wrong_height(&anyone, dr_id, "0xcommitment".hash())
        .unwrap();
}

#[test]
fn reveal_result() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&alice, dr_id, alice_reveal.hash()).unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&bob, dr_id, bob_reveal.hash()).unwrap();

    // alice reveals
    test_info.reveal_result(&alice, dr_id, alice_reveal).unwrap();

    let revealed = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&dr_id.to_hex()));
}

#[test]
fn reveals_meet_replication_factor() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&alice, dr_id, alice_reveal.hash()).unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&bob, dr_id, bob_reveal.hash()).unwrap();

    // alice reveals
    test_info.reveal_result(&alice, dr_id, alice_reveal).unwrap();

    // bob reveals
    test_info.reveal_result(&bob, dr_id, bob_reveal).unwrap();

    let resolved = test_info.get_data_result(dr_id);
    assert_eq!(resolved.dr_id, dr_id.to_hex())
}

#[test]
#[should_panic(expected = "RevealNotStarted")]
fn cannot_reveal_if_commit_rf_not_met() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&alice, dr_id, alice_reveal.hash()).unwrap();

    // alice reveals
    test_info.reveal_result(&alice, dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn cannot_reveal_if_user_did_not_commit() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&alice, dr_id, alice_reveal.hash()).unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };

    // bob reveals
    test_info.reveal_result(&bob, dr_id, bob_reveal).unwrap();

    let revealed = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&dr_id.to_hex()));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn cannot_double_reveal() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&alice, dr_id, alice_reveal.hash()).unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&bob, dr_id, bob_reveal.hash()).unwrap();

    // alice reveals
    test_info.reveal_result(&alice, dr_id, alice_reveal.clone()).unwrap();

    // alice reveals again
    test_info.reveal_result(&alice, dr_id, alice_reveal).unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_must_match_commitment() {
    let mut test_info = TestInfo::init();

    // post a data request
    let alice = test_info.new_executor("alice", Some(2));
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info.post_data_request(&alice, dr, vec![], vec![], 1).unwrap();

    // alice commits a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info
        .commit_result(
            &alice,
            dr_id,
            RevealBody {
                salt:      alice.salt(),
                reveal:    "11".hash().into(),
                gas_used:  0u128.into(),
                exit_code: 0,
            }
            .hash(),
        )
        .unwrap();

    // bob also commits
    let bob = test_info.new_executor("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().into(),
        gas_used:  0u128.into(),
        exit_code: 0,
    };
    test_info.commit_result(&bob, dr_id, bob_reveal.hash()).unwrap();

    // alice reveals
    test_info.reveal_result(&alice, dr_id, alice_reveal).unwrap();

    let revealed = test_info.get_data_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&dr_id.to_hex()));
}

// #[test]
// fn post_data_result() {
//     let mut deps = mock_dependencies();
//     let creator = TestExecutor::new("creator", Some(2));
//     instantiate_contract(deps.as_mut(), creator.info()).unwrap();

//     // post a data request
//     let alice = TestExecutor::new("alice", Some(2));
//     let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
//     test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

//     // commit a data result
//     let alice_reveal = RevealBody {
//         salt:      alice.salt(),
//         reveal:    "10".hash().to_vec(),
//         gas_used:  0,
//         exit_code: 0,
//     };
//     test_helpers::commit_result(
//         deps.as_mut(),
//         alice.info(),
//         &alice,
//         constructed_dr_id,
//         alice_reveal.hash(),
//         None,
//         None,
//     )
//     .unwrap();

//     // alice reveals
//     test_helpers::reveal_result(
//         deps.as_mut(),
//         alice.info(),
//         &alice,
//         constructed_dr_id,
//         alice_reveal.clone(),
//         None,
//         None,
//     )
//     .unwrap();

//     // post a data result
//     let result = test_helpers::construct_result(dr_args, alice_reveal, 0, vec![], vec![]);
//     test_helpers::post_data_result(deps.as_mut(), constructed_dr_id, result, 0).unwrap();

//     let resolved = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Resolved);
//     assert_eq!(1, resolved.len());
//     assert!(resolved.contains_key(&constructed_dr_id.to_hex()));
// }

// #[test]
// #[should_panic(expected = "Unauthorized")]
// fn post_data_result_only_sudoers() {
//     let mut deps = mock_dependencies();
//     let creator = TestExecutor::new("creator", Some(2));
//     instantiate_contract(deps.as_mut(), creator.info()).unwrap();

//     // let contract = ContractWrapper::new(
//     //     crate::contract::execute,
//     //     crate::contract::instantiate,
//     //     crate::contract::query,
//     // )
//     // .with_sudo(sudo);

//     // post a data request
//     let alice = TestExecutor::new("alice", Some(2));
//     let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
//     test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

//     // commit a data result
//     let alice_reveal = RevealBody {
//         salt:      alice.salt(),
//         reveal:    "10".hash().to_vec(),
//         gas_used:  0,
//         exit_code: 0,
//     };
//     test_helpers::commit_result(
//         deps.as_mut(),
//         alice.info(),
//         &alice,
//         constructed_dr_id,
//         alice_reveal.hash(),
//         None,
//         None,
//     )
//     .unwrap();

//     // alice reveals
//     test_helpers::reveal_result(
//         deps.as_mut(),
//         alice.info(),
//         &alice,
//         constructed_dr_id,
//         alice_reveal.clone(),
//         None,
//         None,
//     )
//     .unwrap();

//     // post a data result
//     let result = test_helpers::construct_result(dr_args, alice_reveal, 0, vec![], vec![]);
//     test_helpers::post_data_result(deps.as_mut(), constructed_dr_id, result, 0).unwrap();
// }
