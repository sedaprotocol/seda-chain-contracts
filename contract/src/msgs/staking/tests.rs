use super::*;
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

#[test]
fn deposit_stake_withdraw() {
    let mut test_info = TestInfo::init();

    // can't register without depositing tokens
    let mut anyone = test_info.new_executor("anyone", Some(3));

    // register a data request executor
    test_info
        .reg_and_stake(&mut anyone, Some("address".to_string()), 1)
        .unwrap();
    let executor_is_eligible = test_info.is_executor_eligible(anyone.pub_key());
    assert!(executor_is_eligible);

    // data request executor's stake should be 1
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(value.map(|x| x.tokens_staked), Some(1u8.into()),);

    // the data request executor stakes 2 more tokens
    test_info.increase_stake(&mut anyone, 2).unwrap();
    let executor_is_eligible = test_info.is_executor_eligible(anyone.pub_key());
    assert!(executor_is_eligible);
    // data request executor's stake should be 3
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(value.map(|x| x.tokens_staked), Some(3u8.into()),);

    // the data request executor unstakes 1
    let _res = test_info.unstake(&anyone, 1);
    let executor_is_eligible = test_info.is_executor_eligible(anyone.pub_key());
    assert!(executor_is_eligible);
    // data request executor's stake should be 2 and pending 1
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 1u8.into(),
        }),
    );

    // the data request executor withdraws 1
    let _res = test_info.withdraw(&anyone, 1);
    let executor_is_eligible = test_info.is_executor_eligible(anyone.pub_key());
    assert!(executor_is_eligible);

    // data request executor's stake should be 2 and pending 0
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // unstake 2 more
    test_info.unstake(&anyone, 2).unwrap();

    // assert executer is no longer eligible for committe inclusion
    let executor_is_eligible = test_info.is_executor_eligible(anyone.pub_key());
    assert!(!executor_is_eligible);
}

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let mut test_info = TestInfo::init();

    let mut anyone = test_info.new_executor("anyone", None);
    test_info.increase_stake_no_funds(&mut anyone).unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn insufficient_funds() {
    let mut test_info = TestInfo::init();

    // register a data request executor
    let mut alice = test_info.new_executor("alice", Some(1000));
    test_info.reg_and_stake(&mut alice, None, 1).unwrap();

    // try unstaking more than staked
    alice.set_seda(0);
    test_info.unstake(&alice, 2).unwrap();
}

#[test]
fn register_data_request_executor() {
    let mut test_info = TestInfo::init();

    // fetching data request executor for an address that doesn't exist should return None
    let mut anyone = test_info.new_executor("anyone", Some(2));
    let value = test_info.get_staker(anyone.pub_key());
    assert_eq!(value, None);

    // someone registers a data request executor
    test_info
        .reg_and_stake(&mut anyone, Some("memo".to_string()), 1)
        .unwrap();

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

#[test]
fn unregister_data_request_executor() {
    let mut test_info = TestInfo::init();

    // someone registers a data request executor
    let mut anyone = test_info.new_executor("anyone", Some(2));
    test_info
        .reg_and_stake(&mut anyone, Some("memo".to_string()), 2)
        .unwrap();

    // should be able to fetch the data request executor
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // can't unregister the data request executor if it has staked tokens
    let res = test_info.unregister(&anyone);
    assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

    // unstake and withdraw all tokens
    test_info.unstake(&anyone, 2).unwrap();
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             0u8.into(),
            tokens_pending_withdrawal: 2u8.into(),
        }),
    );

    test_info.withdraw(&anyone, 2).unwrap();
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string()),
            tokens_staked:             0u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // unregister the data request executor
    test_info.unregister(&anyone).unwrap();

    // fetching data request executor after unregistering should return None
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());

    assert_eq!(value, None);
}
