use cosmwasm_std::testing::{mock_dependencies, mock_info};

use super::*;
use crate::instantiate_contract;

#[test]
fn post_data_request() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &coins(2, "token"));

    instantiate_contract(deps.as_mut(), info.clone()).unwrap();

    // data request with id 0x69... does not yet exist
    let value: GetDataRequestResponse = get_dr(
        deps.as_mut(),
        "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".simple_hash(),
    );
    assert_eq!(None, value.value);

    let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

    let info = mock_info("anyone", &coins(2, "token"));

    let msg = ExecuteMsg::PostDataRequest {
        posted_dr:       dr_args,
        seda_payload:    vec![],
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
