use cosmwasm_std::MessageInfo;

use super::helpers::*;
use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};

use crate::contract::execute;

use common::error::ContractError;
use common::msg::DataRequestsExecuteMsg as ExecuteMsg;
use common::types::SimpleHash;
use cosmwasm_std::coins;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
#[test]
fn post_data_request() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &coins(2, "token"));

    instantiate_dr_contract(deps.as_mut(), info.clone()).unwrap();

    // data request with id 0x69... does not yet exist
    let value: GetDataRequestResponse = get_dr(
        deps.as_mut(),
        "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".simple_hash(),
    );
    assert_eq!(None, value.value);

    let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

    let info = mock_info("anyone", &coins(2, "token"));

    let msg = ExecuteMsg::PostDataRequest {
        posted_dr: dr_args,
        seda_payload: vec![],
        payback_address: vec![],
    };
    // someone posts a data request
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // expect an error when trying to post it again
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value: GetDataRequestResponse = get_dr(deps.as_mut(), constructed_dr_id);

    let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

    assert_eq!(
        Some(construct_dr(constructed_dr_id, dr_args, vec![])),
        received_value.value
    );

    // nonexistent data request does not yet exist

    let value: GetDataRequestResponse = get_dr(deps.as_mut(), "nonexistent".simple_hash());

    assert_eq!(None, value.value);
}

#[test]
fn get_data_requests() {
    let mut deps = mock_dependencies();
    let info: MessageInfo = mock_info("creator", &coins(2, "token"));

    instantiate_dr_contract(deps.as_mut(), info).unwrap();

    let (_, dr_args1) = calculate_dr_id_and_args(1, 3);

    let (_, dr_args2) = calculate_dr_id_and_args(2, 3);

    let (_, dr_args3) = calculate_dr_id_and_args(3, 3);

    // someone posts three data requests
    let info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::PostDataRequest {
        posted_dr: dr_args1,
        seda_payload: vec![],
        payback_address: vec![],
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::PostDataRequest {
        posted_dr: dr_args2,
        seda_payload: vec![],
        payback_address: vec![],
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::PostDataRequest {
        posted_dr: dr_args3,
        seda_payload: vec![],
        payback_address: vec![],
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let (constructed_dr_id1, dr_args1) = calculate_dr_id_and_args(1, 3);

    let (constructed_dr_id2, dr_args2) = calculate_dr_id_and_args(2, 3);

    let (constructed_dr_id3, dr_args3) = calculate_dr_id_and_args(3, 3);

    let constructed_dr1 = construct_dr(constructed_dr_id1, dr_args1, vec![]);
    let constructed_dr2 = construct_dr(constructed_dr_id2, dr_args2, vec![]);
    let constructed_dr3 = construct_dr(constructed_dr_id3, dr_args3, vec![]);

    // fetch all three data requests

    let response: GetDataRequestsFromPoolResponse = get_drs_from_pool(deps.as_mut(), None, None);

    assert_eq!(
        GetDataRequestsFromPoolResponse {
            value: vec![
                constructed_dr1.clone(),
                constructed_dr2.clone(),
                constructed_dr3.clone(),
            ]
        },
        response
    );

    // fetch data requests with limit of 2

    let response: GetDataRequestsFromPoolResponse = get_drs_from_pool(deps.as_mut(), None, Some(2));

    assert_eq!(
        GetDataRequestsFromPoolResponse {
            value: vec![constructed_dr1.clone(), constructed_dr2.clone(),]
        },
        response
    );

    // fetch a single data request

    let response: GetDataRequestsFromPoolResponse =
        get_drs_from_pool(deps.as_mut(), Some(1), Some(1));

    assert_eq!(
        GetDataRequestsFromPoolResponse {
            value: vec![constructed_dr2.clone()]
        },
        response
    );

    // fetch all data requests starting from id 1

    let response: GetDataRequestsFromPoolResponse = get_drs_from_pool(deps.as_mut(), Some(1), None);

    assert_eq!(
        GetDataRequestsFromPoolResponse {
            value: vec![constructed_dr2.clone(), constructed_dr3.clone(),]
        },
        response
    );
}

#[test]
fn test_hash_data_request() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &coins(2, "token"));

    // instantiate contract
    instantiate_dr_contract(deps.as_mut(), info).unwrap();

    let (constructed_dr_id, _) = calculate_dr_id_and_args(1, 3);

    println!("0x{}", hex::encode(constructed_dr_id));
}
