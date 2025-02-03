use msgs::data_requests::{
    sudo::{DistributionDataProxyReward, DistributionMessage},
    RevealBody,
};
use seda_common::msgs::staking::{Staker, StakingConfig};

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

    let new_config = StakingConfig {
        minimum_stake_to_register:               1u8.into(),
        minimum_stake_for_committee_eligibility: 1u8.into(),
        allowlist_enabled:                       true,
    };

    // owner sets staking config
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();

    test_info
        .add_to_allowlist(&test_info.creator(), anyone.pub_key())
        .unwrap();

    // register a data request executor
    test_info.stake(&mut anyone, Some("address".to_string()), 1).unwrap();
    let is_executor_committee_eligible = test_info.is_staker_executor(&anyone);
    assert!(is_executor_committee_eligible);

    // data request executor's stake should be 1
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(value.map(|x| x.tokens_staked), Some(1u8.into()),);

    // the data request executor stakes 2 more tokens
    test_info.stake(&mut anyone, Some("address".to_string()), 2).unwrap();
    let is_executor_committee_eligible = test_info.is_staker_executor(&anyone);
    assert!(is_executor_committee_eligible);

    // data request executor's stake should be 3
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(value.map(|x| x.tokens_staked), Some(3u8.into()),);

    // the data request executor unstakes 1
    let _res = test_info.unstake(&anyone, 1);
    let is_executor_committee_eligible = test_info.is_staker_executor(&anyone);
    assert!(is_executor_committee_eligible);

    // data request executor's stake should be 2 and pending 1
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".as_bytes().into()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 1u8.into(),
        }),
    );

    // the data request executor withdraws 1
    let _res = test_info.withdraw(&mut anyone, 1);
    let is_executor_committee_eligible = test_info.is_staker_executor(&anyone);
    assert!(is_executor_committee_eligible);

    // data request executor's stake should be 2 and pending 0
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".to_string().as_bytes().into()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // unstake 2 more
    test_info.unstake(&anyone, 2).unwrap();

    // assert executor is no longer eligible for committee inclusion
    let is_executor_committee_eligible = test_info.is_staker_executor(&anyone);
    assert!(!is_executor_committee_eligible);
}

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let mut test_info = TestInfo::init();

    let mut anyone = test_info.new_executor("anyone", None);
    test_info
        .stake_with_no_funds(&mut anyone, Some("address".to_string()))
        .unwrap();
}

#[test]
#[should_panic(expected = "InsufficientFunds")]
fn insufficient_funds() {
    let mut test_info = TestInfo::init();

    // register a data request executor
    let mut alice = test_info.new_executor("alice", Some(1000));
    test_info.stake(&mut alice, None, 1).unwrap();

    // try unstaking more than staked
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
    test_info.stake(&mut anyone, Some("memo".to_string()), 1).unwrap();

    // should be able to fetch the data request executor
    let value = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string().as_bytes().into()),
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
    test_info.stake(&mut anyone, Some("memo".to_string()), 2).unwrap();

    // should be able to fetch the data request executor
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string().as_bytes().into()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // can't unregister the data request executor if it has staked tokens
    // let res = test_info.unregister(&anyone);
    // assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

    // unstake and withdraw all tokens
    test_info.unstake(&anyone, 2).unwrap();
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string().as_bytes().into()),
            tokens_staked:             0u8.into(),
            tokens_pending_withdrawal: 2u8.into(),
        }),
    );

    // unregister the data request executor by withdrawing all funds
    test_info.withdraw(&mut anyone, 2).unwrap();

    // fetching data request executor after unregistering should return None
    let value: Option<Staker> = test_info.get_staker(anyone.pub_key());

    assert_eq!(value, None);
}

#[test]
fn executor_eligible() {
    let mut test_info = TestInfo::init();

    // someone registers a data request executor
    let mut anyone = test_info.new_executor("anyone", Some(40));
    test_info.stake(&mut anyone, Some("memo".to_string()), 2).unwrap();

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_executor_eligible = test_info.is_executor_eligible(&anyone, dr_id);
    assert!(is_executor_eligible);
}

#[test]
fn multiple_executor_eligible() {
    let mut test_info = TestInfo::init();

    // someone registers a data request executor
    let mut val1 = test_info.new_executor("val1", Some(40));
    test_info.stake(&mut val1, Some("memo".to_string()), 2).unwrap();

    let mut val2 = test_info.new_executor("val2", Some(20));
    test_info.stake(&mut val2, Some("memo".to_string()), 2).unwrap();

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut val1, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_val1_executor_eligible = test_info.is_executor_eligible(&val1, dr_id.to_string());
    let is_val2_executor_eligible = test_info.is_executor_eligible(&val2, dr_id);

    if is_val1_executor_eligible && is_val2_executor_eligible {
        panic!("Both validators cannot be chosen at the same time");
    }

    assert!(is_val1_executor_eligible || is_val2_executor_eligible);
}

#[test]
fn multiple_executor_eligible_exact_replication_factor() {
    let mut test_info = TestInfo::init();

    // someone registers a data request executor
    let mut val1 = test_info.new_executor("val1", Some(40));
    test_info.stake(&mut val1, Some("memo".to_string()), 2).unwrap();

    let mut val2 = test_info.new_executor("val2", Some(20));
    test_info.stake(&mut val2, Some("memo".to_string()), 2).unwrap();

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info
        .post_data_request(&mut val1, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_val1_executor_eligible = test_info.is_executor_eligible(&val1, dr_id.to_string());
    let is_val2_executor_eligible = test_info.is_executor_eligible(&val2, dr_id);

    assert!(is_val1_executor_eligible && is_val2_executor_eligible);
}

#[test]
fn only_allow_active_stakers_to_be_eligible() {
    let mut test_info = TestInfo::init();

    let msg = msgs::ExecuteMsg::Staking(msgs::staking::execute::ExecuteMsg::SetStakingConfig(StakingConfig {
        minimum_stake_to_register:               Uint128::from(10u32),
        minimum_stake_for_committee_eligibility: Uint128::from(20u32),
        allowlist_enabled:                       false,
    }));

    test_info.execute::<()>(&test_info.creator(), &msg).unwrap();

    // someone registers a data request executor
    let mut val1 = test_info.new_executor("val1", Some(80));
    test_info.stake(&mut val1, Some("memo".to_string()), 21).unwrap();

    let mut val2 = test_info.new_executor("val2", Some(20));
    test_info.stake(&mut val2, Some("memo".to_string()), 10).unwrap();

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = test_info
        .post_data_request(&mut val1, dr.clone(), vec![], vec![1, 2, 3], 2, None)
        .unwrap();

    // perform the check
    let is_val1_executor_eligible = test_info.is_executor_eligible(&val1, dr_id.to_string());
    let is_val2_executor_eligible = test_info.is_executor_eligible(&val2, dr_id);

    assert!(is_val1_executor_eligible);
    assert!(!is_val2_executor_eligible);
}

const VALIDATORS_AMOUNT: usize = 50;

lazy_static::lazy_static! {
    static ref LARGE_SET_VALIDATOR_NAMES: Vec<String> = {
        let mut names = Vec::new();
        for i in 0..VALIDATORS_AMOUNT {
            names.push(format!("validator_{}", i));
        }
        names
    };
}

#[test]
fn large_set_executor_eligible() {
    let mut test_info = TestInfo::init();
    let mut validators = Vec::with_capacity(VALIDATORS_AMOUNT);
    for validator_name in LARGE_SET_VALIDATOR_NAMES.iter() {
        let mut validator = test_info.new_executor(validator_name, Some(20));
        test_info.stake(&mut validator, Some("memo".to_string()), 2).unwrap();
        validators.push(validator);
    }

    // someone registers a data request executor
    let mut anyone = test_info.new_executor("anyone", Some(40));
    let replication_factor = 8;

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, replication_factor);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    let mut amount_eligible = 0;

    for validator in validators {
        let is_eligible = test_info.is_executor_eligible(&validator, dr_id.to_string());

        if is_eligible {
            amount_eligible += 1;
        }
    }

    assert_eq!(amount_eligible, replication_factor);
}

#[test]
fn executor_not_eligible_if_dr_resolved() {
    let mut test_info = TestInfo::init();

    // someone registers a data request executor
    let mut anyone = test_info.new_executor("anyone", Some(40));
    test_info.stake(&mut anyone, Some("memo".to_string()), 2).unwrap();

    // post a data request
    let dr = data_requests::test::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = test_info
        .post_data_request(&mut anyone, dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    let reveal = RevealBody {
        id:                dr_id.clone(),
        salt:              anyone.salt(),
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![],
    };
    // commit
    test_info
        .commit_result(&anyone, &dr_id, reveal.try_hash().unwrap())
        .unwrap();
    // reveal
    test_info.reveal_result(&anyone, &dr_id, reveal.clone()).unwrap();

    // Owner removes the data request
    test_info
        .remove_data_request(
            dr_id.clone(),
            vec![DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                to:     Binary::new(anyone.addr().as_bytes().to_vec()),
                amount: 10u128.into(),
            })],
        )
        .unwrap();

    // perform the check
    let is_executor_eligible = test_info.is_executor_eligible(&anyone, dr_id);
    assert!(!is_executor_eligible);
}

#[test]
fn execute_messages_get_paused() {
    let mut test_info = TestInfo::init();

    // register a data request executor that can try to unstake after pausing
    let mut alice = test_info.new_executor("alice", Some(100));
    test_info.stake(&mut alice, None, 10).unwrap();

    // pause the contract
    test_info.pause(&test_info.creator()).unwrap();
    assert!(test_info.is_paused());

    // try to have a new staker register
    let mut bob = test_info.new_executor("bob", Some(100));
    let res = test_info.stake(&mut bob, None, 10);
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // try to have an existing staker unstake
    let res = test_info.unstake(&alice, 10);
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // try to have an existing staker withdraw rewards
    let res = test_info.withdraw(&mut alice, 10);
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // can still change the staking config
    let new_config = StakingConfig {
        minimum_stake_to_register:               10u8.into(),
        minimum_stake_for_committee_eligibility: 20u8.into(),
        allowlist_enabled:                       false,
    };
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();
}

#[test]
fn staker_not_in_allowlist_withdrawing() {
    let mut test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake_to_register:               10u8.into(),
        minimum_stake_for_committee_eligibility: 10u8.into(),
        allowlist_enabled:                       true,
    };
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();
    let mut alice = test_info.new_executor("alice", Some(100));

    // add alice to the allowlist
    test_info
        .add_to_allowlist(&test_info.creator(), alice.pub_key())
        .unwrap();

    // now alice can register a data request executor
    test_info.stake(&mut alice, None, 10).unwrap();

    // remove alice from the allowlist
    test_info
        .remove_from_allowlist(&test_info.creator(), alice.pub_key())
        .unwrap();

    // alice withdraws most her funds but should still be in stakers map
    test_info.withdraw(&mut alice, 9).unwrap();
    let stake_info = test_info.get_staker(alice.pub_key()).unwrap();
    assert_eq!(stake_info.tokens_pending_withdrawal, Uint128::new(1));

    // alice withdraws the rest of her funds and should be removed from stakers map
    test_info.withdraw(&mut alice, 1).unwrap();
    let stake_info = test_info.get_staker(alice.pub_key());
    assert_eq!(stake_info, None);
}

#[test]
fn minimum_stake_cannot_be_zero() {
    let mut test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake_to_register:               0u8.into(),
        minimum_stake_for_committee_eligibility: 10u8.into(),
        allowlist_enabled:                       true,
    };
    let res = test_info.set_staking_config(&test_info.creator(), new_config);
    assert!(res.is_err_and(|x| x == ContractError::ZeroMinimumStakeToRegister));
}

#[test]
fn minimum_stake_for_committee_eligibility_cannot_be_zero() {
    let mut test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake_to_register:               10u8.into(),
        minimum_stake_for_committee_eligibility: 0u8.into(),
        allowlist_enabled:                       true,
    };
    let res = test_info.set_staking_config(&test_info.creator(), new_config);
    assert!(res.is_err_and(|x| x == ContractError::ZeroMinimumStakeForCommitteeEligibility));
}
