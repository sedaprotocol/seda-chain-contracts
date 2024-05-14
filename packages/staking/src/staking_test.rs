use common::{
    error::ContractError,
    msg::{GetStaker, StakingExecuteMsg as ExecuteMsg},
    state::Staker,
    test_utils::TestExecutor,
};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_env, mock_info},
};

use super::helpers::*;
use crate::{contract::execute, state::ELIGIBLE_DATA_REQUEST_EXECUTORS};

#[test]
fn deposit_stake_withdraw() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(0, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    // cant register without depositing tokens
    let info = mock_info("anyone", &coins(0, "token"));
    let exec = TestExecutor::new("anyone");

    let res = helper_reg_and_stake(deps.as_mut(), info, &exec, None);
    assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

    // register a data request executor
    let info = mock_info("anyone", &coins(1, "token"));

    let _res = helper_reg_and_stake(deps.as_mut(), info.clone(), &exec, Some("address".to_string()));
    let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
        .load(&deps.storage, &exec.public_key) // Convert Addr to Vec<u8>
        .unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 1
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("address".to_string()),
                tokens_staked:             1,
                tokens_pending_withdrawal: 0,
            }),
        }
    );

    // the data request executor stakes 2 more tokens
    let info = mock_info("anyone", &coins(2, "token"));
    let _res = helper_increase_stake(deps.as_mut(), info.clone(), &exec).unwrap();
    let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
        .load(&deps.storage, &exec.public_key)
        .unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 3
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("address".to_string()),
                tokens_staked:             3,
                tokens_pending_withdrawal: 0,
            }),
        }
    );

    // the data request executor unstakes 1
    let info = mock_info("anyone", &coins(0, "token"));

    let _res = helper_unstake(deps.as_mut(), info.clone(), &exec, 1);
    let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
        .load(&deps.storage, &exec.public_key)
        .unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 1 and pending 1
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("address".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 1,
            }),
        }
    );

    // the data request executor withdraws 1
    let info = mock_info("anyone", &coins(0, "token"));
    let _res = helper_withdraw(deps.as_mut(), info.clone(), &exec, 1);

    let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
        .load(&deps.storage, &exec.public_key)
        .unwrap();
    assert!(executor_is_eligible);

    // data request executor's stake should be 1 and pending 0
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("address".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 0,
            }),
        }
    );

    // unstake 2 more
    helper_unstake(deps.as_mut(), info, &exec, 2).unwrap();

    // assert executer is no longer eligible for committe inclusion
    let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS.has(&deps.storage, &exec.public_key);
    assert!(!executor_is_eligible);
}

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();
    let exec = TestExecutor::new("anyone");

    let msg = ExecuteMsg::IncreaseStake {
        signature: exec.sign(["deposit_and_stake".as_bytes().to_vec()]),
    };
    let info = mock_info("anyone", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn insufficient_funds() {
    let mut deps = mock_dependencies();

    let info = mock_info("alice", &coins(1, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info.clone()).unwrap();
    let alice = TestExecutor::new("alice");

    // register a data request executor
    helper_reg_and_stake(deps.as_mut(), info.clone(), &alice, None).unwrap();

    // try unstaking more than staked
    let info = mock_info("alice", &coins(0, "token"));
    helper_unstake(deps.as_mut(), info.clone(), &alice, 2).unwrap();
}

#[test]
fn register_data_request_executor() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    let exec = TestExecutor::new("anyone");
    // fetching data request executor for an address that doesn't exist should return None
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(value, GetStaker { value: None });

    // someone registers a data request executor
    let info = mock_info("anyone", &coins(2, "token"));

    let _res = helper_reg_and_stake(deps.as_mut(), info, &exec, Some("memo".to_string())).unwrap();

    // should be able to fetch the data request executor

    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());
    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("memo".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 0,
            }),
        }
    );
}

#[test]
fn unregister_data_request_executor() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    // someone registers a data request executor
    let info = mock_info("anyone", &coins(2, "token"));
    let exec = TestExecutor::new("anyone");

    let _res = helper_reg_and_stake(deps.as_mut(), info, &exec, Some("memo".to_string())).unwrap();

    // should be able to fetch the data request executor
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetStaker {
            value: Some(Staker {
                memo:                      Some("memo".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 0,
            }),
        }
    );

    // can't unregister the data request executor if it has staked tokens
    let info = mock_info("anyone", &coins(2, "token"));
    let res = helper_unregister(deps.as_mut(), info, &exec);
    assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

    // unstake and withdraw all tokens
    let info = mock_info("anyone", &coins(0, "token"));

    let _res = helper_unstake(deps.as_mut(), info.clone(), &exec, 2);
    let info = mock_info("anyone", &coins(0, "token"));
    let _res = helper_withdraw(deps.as_mut(), info.clone(), &exec, 2);

    // unregister the data request executor
    let info = mock_info("anyone", &coins(2, "token"));
    let _res = helper_unregister(deps.as_mut(), info, &exec).unwrap();

    // fetching data request executor after unregistering should return None
    let value: GetStaker = get_staker(deps.as_mut(), exec.public_key.clone());

    assert_eq!(value, GetStaker { value: None });
}
