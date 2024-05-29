use cosmwasm_std::testing::mock_dependencies;
use tests::data_requests::test_helpers;

use super::*;
use crate::{instantiate_contract, TestExecutor};

#[test]
fn post_data_request() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // data request with id 0x69... does not yet exist
    let value = test_helpers::get_dr(
        deps.as_mut(),
        "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".hash(),
    );
    assert_eq!(None, value);

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 3);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // expect an error when trying to post it again
    let res = test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = test_helpers::get_dr(deps.as_mut(), constructed_dr_id);
    assert_eq!(
        Some(test_helpers::construct_dr(constructed_dr_id, dr_args, vec![])),
        received_value
    );
    let await_commits = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Committing);
    assert_eq!(1, await_commits.len());
    assert!(await_commits.contains_key(&constructed_dr_id.to_hex()));

    // nonexistent data request does not yet exist
    let value = test_helpers::get_dr(deps.as_mut(), "nonexistent".hash());
    assert_eq!(None, value);
}

#[test]
fn commit_result() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 3);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();

    let commiting = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Committing);
    assert_eq!(1, commiting.len());
    assert!(commiting.contains_key(&constructed_dr_id.to_hex()));
}

#[test]
fn commits_meet_replication_factor() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();

    let commiting = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Revealing);
    assert_eq!(1, commiting.len());
    assert!(commiting.contains_key(&constructed_dr_id.to_hex()));
}

#[test]
#[should_panic(expected = "AlreadyCommitted")]
fn cannot_double_commit() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();

    // commit again as the same user
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "RevealStarted")]
fn cannot_commit_after_replication_factor_reached() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();

    // commit again as a different user
    let new = TestExecutor::new("new", Some(2));
    test_helpers::commit_result(
        deps.as_mut(),
        new.info(),
        &new,
        constructed_dr_id,
        "0xcommitment".hash(),
        None,
        None,
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "InvalidProof")]
fn commits_wrong_signature_fails() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let anyone = TestExecutor::new("anyone", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
    test_helpers::post_data_request(deps.as_mut(), anyone.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    test_helpers::commit_result(
        deps.as_mut(),
        anyone.info(),
        &anyone,
        constructed_dr_id,
        "0xcommitment".hash(),
        Some(9),
        Some(10),
    )
    .unwrap();
}

#[test]
fn reveal_result() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // bob also commits
    let bob = TestExecutor::new("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // alice reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal,
        None,
        None,
    )
    .unwrap();

    let revealed = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Revealing);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&constructed_dr_id.to_hex()));
}

#[test]
fn reveals_meet_replication_factor() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // bob also commits
    let bob = TestExecutor::new("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // alice reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal,
        None,
        None,
    )
    .unwrap();

    // bob reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal,
        None,
        None,
    )
    .unwrap();

    // TODO this should check if status is Tallying.
    // but for now we mock the tallying so its set to Resolved
    let resolved = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Resolved);
    assert_eq!(1, resolved.len());
    assert!(resolved.contains_key(&constructed_dr_id.to_hex()));
}

#[test]
#[should_panic(expected = "RevealNotStarted")]
fn cannot_reveal_if_commit_rf_not_met() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // alice reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal,
        None,
        None,
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "NotCommitted")]
fn cannot_reveal_if_user_did_not_commit() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 1);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // bob also commits
    let bob = TestExecutor::new("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };

    // bob reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal,
        None,
        None,
    )
    .unwrap();

    let revealed = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Revealing);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&constructed_dr_id.to_hex()));
}

#[test]
#[should_panic(expected = "AlreadyRevealed")]
fn cannot_double_reveal() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // bob also commits
    let bob = TestExecutor::new("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // alice reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal.clone(),
        None,
        None,
    )
    .unwrap();

    // alice reveals again
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal,
        None,
        None,
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "RevealMismatch")]
fn reveal_must_match_commitment() {
    let mut deps = mock_dependencies();
    let creator = TestExecutor::new("creator", Some(2));
    instantiate_contract(deps.as_mut(), creator.info()).unwrap();

    // post a data request
    let alice = TestExecutor::new("alice", Some(2));
    let (constructed_dr_id, dr_args) = test_helpers::calculate_dr_id_and_args(1, 2);
    test_helpers::post_data_request(deps.as_mut(), alice.info(), dr_args.clone(), vec![], vec![]).unwrap();

    // commit a data result
    let alice_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "10".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        RevealBody {
            salt:      alice.salt(),
            reveal:    "11".hash().to_vec(),
            gas_used:  0,
            exit_code: 0,
        }
        .hash(),
        None,
        None,
    )
    .unwrap();

    // bob also commits
    let bob = TestExecutor::new("bob", Some(2));
    let bob_reveal = RevealBody {
        salt:      alice.salt(),
        reveal:    "20".hash().to_vec(),
        gas_used:  0,
        exit_code: 0,
    };
    test_helpers::commit_result(
        deps.as_mut(),
        bob.info(),
        &bob,
        constructed_dr_id,
        bob_reveal.hash(),
        None,
        None,
    )
    .unwrap();

    // alice reveals
    test_helpers::reveal_result(
        deps.as_mut(),
        alice.info(),
        &alice,
        constructed_dr_id,
        alice_reveal,
        None,
        None,
    )
    .unwrap();

    let revealed = test_helpers::get_data_requests_by_status(deps.as_mut(), DataRequestStatus::Revealing);
    assert_eq!(1, revealed.len());
    assert!(revealed.contains_key(&constructed_dr_id.to_hex()));
}
