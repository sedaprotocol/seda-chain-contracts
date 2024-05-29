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
    let await_commits = state::requests_by_status(&deps.storage, &DataRequestStatus::Committing).unwrap();
    assert_eq!(1, await_commits.len());
    assert!(await_commits.contains(&constructed_dr_id));

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

    let commiting = state::requests_by_status(&deps.storage, &DataRequestStatus::Committing).unwrap();
    assert_eq!(1, commiting.len());
    assert!(commiting.contains(&constructed_dr_id));
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

    let commiting = state::requests_by_status(&deps.storage, &DataRequestStatus::Revealing).unwrap();
    assert_eq!(1, commiting.len());
    assert!(commiting.contains(&constructed_dr_id));
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

    state::requests_by_status(&deps.storage, &DataRequestStatus::Committing).unwrap();

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

    state::requests_by_status(&deps.storage, &DataRequestStatus::Committing).unwrap();

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
