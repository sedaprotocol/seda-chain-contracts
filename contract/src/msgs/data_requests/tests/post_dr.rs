use cosmwasm_std::Uint128;
use seda_common::{msgs::data_requests::DataRequestStatus, types::Hash};

use crate::{
    error::ContractError,
    msgs::data_requests::{state::DR_ESCROW, test_helpers},
    types::FromHexStr,
    TestInfo,
};

#[test]
fn post_data_request() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(52), Some(1));

    // data request... does not yet exist
    let value = test_info.get_data_request("673842e9aaa751cb7430630a8706b6d8e6280f3ab8d06cb45c44d57738988236");
    assert_eq!(None, value);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // Expect the dr staked to exist and be correct
    let staked = DR_ESCROW
        .load(
            &*test_info.app().contract_storage(&test_info.contract_addr()),
            &Hash::from_hex_str(&dr_id).unwrap(),
        )
        .unwrap();
    assert_eq!(20, staked.amount.u128());
    assert_eq!(anyone.addr(), staked.poster);

    // expect an error when trying to post it again
    let res = test_info.post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = test_info.get_data_request(&dr_id);
    assert_eq!(Some(test_helpers::construct_dr(dr, vec![], 1)), received_value);
    let await_commits = test_info.get_data_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert!(!await_commits.is_paused);
    assert_eq!(1, await_commits.data_requests.len());
    assert!(await_commits.data_requests.iter().any(|r| r.id == dr_id));

    // nonexistent data request does not yet exist
    let value = test_info.get_data_request("00f0f00f0f00f0f0000000f0fff0ff0ff0ffff0fff00000f000ff000000f000f");
    assert_eq!(None, value);
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn post_dr_with_not_enough_funds_fails() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(22), Some(1));

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.exec_gas_limit = 1000;
    test_info
        .post_data_request(&mut anyone, dr, vec![], vec![], 2, None)
        .unwrap();
}

#[test]
fn post_dr_with_max_gas_limits() {
    let mut test_info = TestInfo::init();
    let mut anyone = test_info.new_executor("anyone", Some(u128::MAX), Some(1));

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    // set gas price to 1 to make the gas limit calculation easier
    dr.gas_price = Uint128::from(1u128);
    dr.exec_gas_limit = u64::MAX;
    dr.tally_gas_limit = u64::MAX;
    test_info
        .post_data_request(
            &mut anyone,
            dr,
            vec![],
            vec![],
            2,
            Some(u128::from(u64::MAX) + u128::from(u64::MAX)),
        )
        .unwrap();
}

#[test]
fn post_data_request_replication_factor_too_high() {
    let mut test_info = TestInfo::init();
    let mut sender = test_info.new_executor("sender", Some(42), None);
    test_info.new_executor("alice", Some(2), Some(1));

    // post a data request with rf=1
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let res = test_info.post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_ok());

    // post a data request with rf=2
    // expect an error when trying to post it again
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let res = test_info.post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestReplicationFactorTooHigh(1)));
}

#[test]
#[should_panic(expected = "DataRequestReplicationFactorZero")]
fn post_data_request_replication_factor_zero() {
    let mut test_info = TestInfo::init();
    let mut sender = test_info.new_executor("sender", Some(22), None);

    // post a data request with rf=0
    let dr = test_helpers::calculate_dr_id_and_args(1, 0);
    test_info
        .post_data_request(&mut sender, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();
}
