use super::{utils::*, *};
use crate::TestInfo;

#[test]
fn owner_set_staking_config() {
    let mut test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake_to_register:               200u8.into(),
        minimum_stake_for_committee_eligibility: 100u8.into(),
        allowlist_enabled:                       false,
    };

    // owner sets staking config
    let res = test_info.set_staking_config(&test_info.creator(), new_config);
    assert!(res.is_ok());
}

#[test]
fn non_owner_cannot_set_staking_config() {
    let mut test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake_to_register:               200u8.into(),
        minimum_stake_for_committee_eligibility: 100u8.into(),
        allowlist_enabled:                       false,
    };

    // non-owner sets staking config
    let non_owner = test_info.new_executor("non-owner", Some(2));
    let res = test_info.set_staking_config(&non_owner, new_config);
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));
}

// #[test]
// fn deposit_stake_withdraw() {
//     let mut deps = mock_dependencies();

//     let creator = mock_info("creator", &coins(0, "token"));
//     let _res = instantiate_contract(deps.as_mut(), creator).unwrap();

//     // cant register without depositing tokens
//     let mut anyone = TestExecutor::new("anyone", Some(0));

//     let res = test_info.reg_and_stake(deps.as_mut(), anyone.info(), &anyone, None);
//     assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

//     // register a data request executor
//     anyone.set_amount(1);
//     let _res = test_info.reg_and_stake(deps.as_mut(), anyone.info(), &anyone, Some("address".to_string()));
//     let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
//     assert!(executor_is_eligible);
//     // data request executor's stake should be 1
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

// assert_eq!(
//     value,
//     Some(Staker {
//         memo:                      Some("address".to_string()),
//         tokens_staked:             1u8.into(),
//         tokens_pending_withdrawal: 0u8.into(),
//     }),
// );

//     // the data request executor stakes 2 more tokens
//     anyone.set_amount(2);
//     let _res = test_info.increase_stake(deps.as_mut(), anyone.info(), &anyone).unwrap();
//     let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
//     assert!(executor_is_eligible);
//     // data request executor's stake should be 3
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

// assert_eq!(
//     value,
//     Some(Staker {
//         memo:                      Some("address".to_string()),
//         tokens_staked:             3u8.into(),
//         tokens_pending_withdrawal: 0u8.into(),
//     }),
// );

//     // the data request executor unstakes 1
//     anyone.set_amount(0);
//     let _res = test_info.unstake(deps.as_mut(), anyone.info(), &anyone, 1);
//     let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
//     assert!(executor_is_eligible);
//     // data request executor's stake should be 1 and pending 1
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

// assert_eq!(
//     value,
//     Some(Staker {
//         memo:                      Some("address".to_string()),
//         tokens_staked:             2u8.into(),
//         tokens_pending_withdrawal: 1u8.into(),
//     }),
// );

//     // the data request executor withdraws 1
//     // anyone.set_amount(0);
//     let _res = test_info.withdraw(deps.as_mut(), anyone.info(), &anyone, 1);
//     let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
//     assert!(executor_is_eligible);

//     // data request executor's stake should be 1 and pending 0
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

// assert_eq!(
//     value,
//     Some(Staker {
//         memo:                      Some("address".to_string()),
//         tokens_staked:             2u8.into(),
//         tokens_pending_withdrawal: 0u8.into(),
//     }),
// );

//     // unstake 2 more
//     test_info.unstake(deps.as_mut(), anyone.info(), &anyone, 2).unwrap();

//     // assert executer is no longer eligible for committe inclusion
//     let executor_is_eligible = is_executor_eligible(deps.as_ref(), anyone.pub_key()).unwrap();
//     assert!(!executor_is_eligible);
// }

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let mut test_info = TestInfo::init();

    let anyone = test_info.new_executor("anyone", None);
    test_info.increase_stake(&anyone).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn insufficient_funds() {
    let mut test_info = TestInfo::init();

    // register a data request executor
    let mut alice = test_info.new_executor("alice", Some(1000));
    test_info.reg_and_stake(&alice, None, 1).unwrap();

    // try unstaking more than staked
    alice.set_amount(0);
    test_info.unstake(&alice, 2).unwrap();
}

#[test]
fn register_data_request_executor() {
    let mut test_info = TestInfo::init();

    // fetching data request executor for an address that doesn't exist should return None
    let anyone = test_info.new_executor("anyone", Some(2));
    let value = test_info.get_staker(anyone.pub_key());
    assert_eq!(value, None);

    // someone registers a data request executor
    test_info.reg_and_stake(&anyone, Some("memo".to_string()), 1).unwrap();

    // should be able to fetch the data request executor
    let value = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             1u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );
}

// #[test]
// fn unregister_data_request_executor() {
//     let mut deps = mock_dependencies();

//     let creator = mock_info("creator", &coins(2, "token"));
//     let _res = instantiate_contract(deps.as_mut(), creator).unwrap();

//     // someone registers a data request executor
//     let mut anyone = TestExecutor::new("anyone", Some(2));

//     let _res = test_info.reg_and_stake(deps.as_mut(), anyone.info(), &anyone, Some("memo".to_string())).unwrap();

//     // should be able to fetch the data request executor
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

// assert_eq!(
//     value,
//     Some(Staker {
//         memo:                      Some("memo".to_string()),
//         tokens_staked:             2u8.into(),
//         tokens_pending_withdrawal: 0u8.into(),
//     }),
// );

//     // can't unregister the data request executor if it has staked tokens
//     let res = test_info.unregister(deps.as_mut(), anyone.info(), &anyone);
//     assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

//     // unstake and withdraw all tokens
//     anyone.set_amount(0);

//     let _res = test_info.unstake(deps.as_mut(), anyone.info(), &anyone, 2);
//     let _res = test_info.withdraw(deps.as_mut(), anyone.info(), &anyone, 2);

//     // unregister the data request executor
//     anyone.set_amount(2);
//     let _res = test_info.unregister(deps.as_mut(), anyone.info(), &anyone).unwrap();

//     // fetching data request executor after unregistering should return None
//     let value: Option<Staker> = test_info.get_staker(deps.as_mut(), anyone.pub_key());

//     assert_eq!(value, None);
// }
