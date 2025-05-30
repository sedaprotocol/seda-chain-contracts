use cosmwasm_std::{Binary, Uint128};
use seda_common::{msgs::data_requests::DataRequestStatus, types::Hash};

use crate::{
    consts::*,
    error::ContractError,
    msgs::data_requests::{
        consts::{min_post_dr_cost, MIN_GAS_PRICE},
        state::DR_ESCROW,
        test_helpers,
    },
    types::FromHexStr,
    TestInfo,
};

#[test]
fn works() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 52, 1);

    // data request... does not yet exist
    let value = anyone.get_data_request("673842e9aaa751cb7430630a8706b6d8e6280f3ab8d06cb45c44d57738988236");
    assert_eq!(None, value);

    // post a data request
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // Expect the dr staked to exist and be correct
    let staked = DR_ESCROW
        .load(
            &*test_info.app().contract_storage(&test_info.contract_addr()),
            &Hash::from_hex_str(&dr_id).unwrap(),
        )
        .unwrap();
    assert_eq!(min_post_dr_cost(), staked.amount.u128());
    assert_eq!(anyone.addr(), staked.poster);

    // expect an error when trying to post it again
    let res = anyone.post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

    // should be able to fetch data request with id 0x69...
    let received_value = anyone.get_data_request(&dr_id);
    assert_eq!(
        Some(test_helpers::construct_dr(dr, vec![], 1).base),
        received_value.map(|dr| dr.base)
    );
    let await_commits = anyone.get_data_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert!(!await_commits.is_paused);
    assert_eq!(1, await_commits.data_requests.len());
    assert!(await_commits.data_requests.iter().any(|r| r.base.id == dr_id));

    // nonexistent data request does not yet exist
    let value = anyone.get_data_request("00f0f00f0f00f0f0000000f0fff0ff0ff0ffff0fff00000f000ff000000f000f");
    assert_eq!(None, value);
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn fails_with_not_enough_funds_fails() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", 1, 1);

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.gas_price = Uint128::new(10_000);
    anyone.post_data_request(dr, vec![], vec![], 2, None).unwrap();
}

#[test]
fn with_max_gas_limits() {
    let test_info = TestInfo::init();
    let anyone = test_info.new_executor("anyone", u128::MAX, 1);

    // post a data request
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    // we can't modify this to be lower than the min...
    // dr.gas_price = Uint128::from(1u128);
    dr.exec_gas_limit = u64::MAX;
    dr.tally_gas_limit = u64::MAX;
    // Can't attach enough funds to pay for this
    anyone
        .post_data_request(
            dr,
            vec![],
            vec![],
            2,
            Some((u128::from(u64::MAX) + u128::from(u64::MAX)) * MIN_GAS_PRICE.u128()),
        )
        .unwrap();
}

#[test]
fn fails_if_request_replication_factor_too_high() {
    let test_info = TestInfo::init();
    let sender = test_info.new_account("sender", 42);
    test_info.new_executor("alice", 2, 1);

    // post a data request with rf=1
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    let res = sender.post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_ok());

    // post a data request with rf=2
    // expect an error when trying to post it again
    let dr = test_helpers::calculate_dr_id_and_args(1, 2);
    let res = sender.post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None);
    assert!(res.is_err_and(|x| x == ContractError::DataRequestReplicationFactorTooHigh(1)));
}

#[test]
#[should_panic(expected = "DataRequestReplicationFactorZero")]
fn fails_if_request_replication_factor_zero() {
    let test_info = TestInfo::init();
    let sender = test_info.new_account("sender", 22);

    // post a data request with rf=0
    let dr = test_helpers::calculate_dr_id_and_args(1, 0);
    sender
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();
}

#[test]
#[should_panic(expected = "GasPriceTooLow")]
fn fails_if_minimum_gas_price_is_not_met() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with gas price = min - 1
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.gas_price -= Uint128::one();
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "ExecGasLimitTooLow")]
fn fails_if_minimum_gas_exec_limit_is_not_met() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with exec gas limit = min - 1
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.exec_gas_limit -= 1;
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "TallyGasLimitTooLow")]
fn fails_if_minimum_gas_tally_limit_is_not_met() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with tally gas limit = min - 1
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.tally_gas_limit -= 1;
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn fails_if_minimum_aseda_not_attached() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with attached funds = min post dr cost - 1
    let dr = test_helpers::calculate_dr_id_and_args(1, 1);
    executor
        .post_data_request(dr, vec![], vec![1, 2, 3], 1, Some(min_post_dr_cost() - 1))
        .unwrap();
}

#[test]
#[should_panic(expected = "ProgramIdInvalidLength")]
fn fails_if_exec_program_id_invalid_length() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with exec program id length != 64
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.exec_program_id = "short".to_string();
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "ProgramIdInvalidLength")]
fn fails_if_tally_program_id_invalid_length() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with tally program id length != 64
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.tally_program_id = "short".to_string();
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "DrFieldTooBig")]
fn fails_if_exec_inputs_too_big() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with exec inputs too big
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.exec_inputs = Binary::new(vec![0; INITIAL_EXEC_INPUT_LIMIT_IN_BYTES.get() as usize + 1]);
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "DrFieldTooBig")]
fn fails_if_tally_inputs_too_big() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with tally inputs too big
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.tally_inputs = Binary::new(vec![0; INITIAL_TALLY_INPUT_LIMIT_IN_BYTES.get() as usize + 1]);
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "DrFieldTooBig")]
fn fails_if_consensus_filter_too_big() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with consensus filter too big
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.consensus_filter = Binary::new(vec![0; INITIAL_CONSENSUS_FILTER_LIMIT_IN_BYTES.get() as usize + 1]);
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}

#[test]
#[should_panic(expected = "DrFieldTooBig")]
fn fails_if_memo_too_big() {
    let test_info = TestInfo::init();
    let executor = test_info.new_executor("sender", 1, 1);

    // post a data request with memo too big
    let mut dr = test_helpers::calculate_dr_id_and_args(1, 1);
    dr.memo = Binary::new(vec![0; INITIAL_MEMO_LIMIT_IN_BYTES.get() as usize + 1]);
    executor.post_data_request(dr, vec![], vec![1, 2, 3], 1, None).unwrap();
}
