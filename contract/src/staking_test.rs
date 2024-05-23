use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_env, mock_info},
};

use super::test_helpers;
use crate::{
    contract::execute,
    error::ContractError,
    msgs::{staking::Staker, StakingExecuteMsg},
    staking::is_executor_eligible,
    test::test_utils::TestExecutor,
};

#[test]
fn deposit_stake_withdraw() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(0, "token"));
    let _res = test_helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();

    // cant register without depositing tokens
    let mut anyone = TestExecutor::new("anyone", Some(0));

    let res = test_helpers::reg_and_stake(deps.as_mut(), anyone.info(), &anyone, None);
    assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

    // register a data request executor
    anyone.set_amount(1);
    let _res = test_helpers::reg_and_stake(deps.as_mut(), anyone.info(), &anyone, Some("address".to_string()));
    let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 1
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             1,
            tokens_pending_withdrawal: 0,
        }),
    );

    // the data request executor stakes 2 more tokens
    anyone.set_amount(2);
    let _res = test_helpers::increase_stake(deps.as_mut(), anyone.info(), &anyone).unwrap();
    let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 3
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             3,
            tokens_pending_withdrawal: 0,
        }),
    );

    // the data request executor unstakes 1
    anyone.set_amount(0);
    let _res = test_helpers::unstake(deps.as_mut(), anyone.info(), &anyone, 1);
    let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
    assert!(executor_is_eligible);
    // data request executor's stake should be 1 and pending 1
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             2,
            tokens_pending_withdrawal: 1,
        }),
    );

    // the data request executor withdraws 1
    // anyone.set_amount(0);
    let _res = test_helpers::withdraw(deps.as_mut(), anyone.info(), &anyone, 1);
    let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
    assert!(executor_is_eligible);

    // data request executor's stake should be 1 and pending 0
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             2,
            tokens_pending_withdrawal: 0,
        }),
    );

    // unstake 2 more
    test_helpers::unstake(deps.as_mut(), anyone.info(), &anyone, 2).unwrap();

    // assert executer is no longer eligible for committe inclusion
    let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
    assert!(!executor_is_eligible);
}

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(2, "token"));
    let _res = test_helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();
    let anyone = TestExecutor::new("anyone", None);

    let msg = StakingExecuteMsg::IncreaseStake {
        signature: anyone.sign(["deposit_and_stake".as_bytes()]),
    };
    execute(deps.as_mut(), mock_env(), anyone.info(), msg.into()).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn insufficient_funds() {
    let mut deps = mock_dependencies();

    let mut alice = TestExecutor::new("alice", Some(1));
    let _res = test_helpers::instantiate_staking_contract(deps.as_mut(), alice.info()).unwrap();

    // register a data request executor
    test_helpers::reg_and_stake(deps.as_mut(), alice.info(), &alice, None).unwrap();

    // try unstaking more than staked
    alice.set_amount(0);
    test_helpers::unstake(deps.as_mut(), alice.info(), &alice, 2).unwrap();
}

#[test]
fn register_data_request_executor() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(2, "token"));
    let _res = test_helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();

    let anyone = TestExecutor::new("anyone", Some(2));
    // fetching data request executor for an address that doesn't exist should return None
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(value, None);

    // someone registers a data request executor
    let _res = test_helpers::reg_and_stake(deps.as_mut(), anyone.info(), &anyone, Some("memo".to_string())).unwrap();

    // should be able to fetch the data request executor

    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             2,
            tokens_pending_withdrawal: 0,
        }),
    );
}

#[test]
fn unregister_data_request_executor() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(2, "token"));
    let _res = test_helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();

    // someone registers a data request executor
    let mut anyone = TestExecutor::new("anyone", Some(2));

    let _res = test_helpers::reg_and_stake(deps.as_mut(), anyone.info(), &anyone, Some("memo".to_string())).unwrap();

    // should be able to fetch the data request executor
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             2,
            tokens_pending_withdrawal: 0,
        }),
    );

    // can't unregister the data request executor if it has staked tokens
    let res = test_helpers::unregister(deps.as_mut(), anyone.info(), &anyone);
    assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

    // unstake and withdraw all tokens
    anyone.set_amount(0);

    let _res = test_helpers::unstake(deps.as_mut(), anyone.info(), &anyone, 2);
    let _res = test_helpers::withdraw(deps.as_mut(), anyone.info(), &anyone, 2);

    // unregister the data request executor
    anyone.set_amount(2);
    let _res = test_helpers::unregister(deps.as_mut(), anyone.info(), &anyone).unwrap();

    // fetching data request executor after unregistering should return None
    let value: Option<Staker> = test_helpers::get_staker(deps.as_mut(), anyone.pub_key());

    assert_eq!(value, None);
}
