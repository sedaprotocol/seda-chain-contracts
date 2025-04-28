use msgs::data_requests::{
    sudo::{DistributionDataProxyReward, DistributionMessage},
    RevealBody,
};
use seda_common::msgs::staking::{Staker, StakingConfig};

use super::*;
use crate::{new_public_key, seda_to_aseda, TestInfo};

#[test]
fn owner_set_staking_config() {
    let test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake:     200u8.into(),
        allowlist_enabled: false,
    };

    // owner sets staking config
    let res = test_info.creator().set_staking_config(new_config);
    assert!(res.is_ok());
}

#[test]
fn non_owner_cannot_set_staking_config() {
    let test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake:     200u8.into(),
        allowlist_enabled: false,
    };

    // non-owner sets staking config
    let non_owner = test_info.new_account("non-owner", 2);
    let res = non_owner.set_staking_config(new_config);
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));
}

#[test]
fn deposit_stake_withdraw() {
    let test_info = TestInfo::init();

    // can't register without depositing tokens
    let anyone = test_info.new_account("anyone", 3);

    let new_config = StakingConfig {
        minimum_stake:     1u8.into(),
        allowlist_enabled: true,
    };

    // owner sets staking config
    test_info.creator().set_staking_config(new_config).unwrap();

    test_info.creator().add_to_allowlist(anyone.pub_key()).unwrap();

    // register a data request executor
    anyone.stake_with_memo(1, "address").unwrap();
    let is_executor_committee_eligible = anyone.is_staker_executor();
    assert!(is_executor_committee_eligible);

    // data request executor's stake should be 1
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(value.map(|x| x.tokens_staked), Some(1u8.into()),);

    // the data request executor stakes 2 more tokens
    anyone.stake_with_memo(2, "address").unwrap();
    let is_executor_eligible = anyone.is_staker_executor();
    assert!(is_executor_eligible);

    // data request executor's stake should be 3
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(value.map(|x| x.tokens_staked), Some(3u8.into()),);

    // the data request executor unstakes
    let _res = anyone.unstake();
    let is_executor_eligible = anyone.is_staker_executor();
    assert!(!is_executor_eligible);

    // can double call unstake. doesn't do anything
    assert!(anyone.unstake().is_ok());

    // data request executor's stake should be 0 and pending 3
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("address".as_bytes().into()),
            tokens_staked:             0u8.into(),
            tokens_pending_withdrawal: 3u8.into(),
        }),
    );

    // the data request executor withdraws their pending tokens
    let _res = anyone.withdraw();
    let is_executor_committee_eligible = anyone.is_staker_executor();
    assert!(!is_executor_committee_eligible);

    // anyone should no longer be a staker since they had 0 stake when withdrawing
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(value, None);

    // assert executor is no longer eligible for committee inclusion
    let is_executor_committee_eligible = anyone.is_staker_executor();
    assert!(!is_executor_committee_eligible);
}

#[test]
#[should_panic(expected = "NoFunds")]
fn no_funds_provided() {
    let test_info = TestInfo::init();

    let anyone = test_info.new_account("anyone", 0);
    anyone.stake_with_no_funds(Some("address".to_string())).unwrap();
}

#[test]
fn register_data_request_executor() {
    let test_info = TestInfo::init();

    // fetching data request executor for an address that doesn't exist should return None
    let anyone = test_info.new_account("anyone", 2);
    let value = anyone.get_staker_info();
    assert_eq!(value, None);

    // someone registers a data request executor
    anyone.stake_with_memo(1, "memo").unwrap();

    // should be able to fetch the data request executor
    let value = anyone.get_staker_info();
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
    let test_info = TestInfo::init();

    // someone registers a data request executor
    let anyone = test_info.new_account("anyone", 2);
    anyone.stake_with_memo(2, "memo").unwrap();

    // should be able to fetch the data request executor
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string().as_bytes().into()),
            tokens_staked:             2u8.into(),
            tokens_pending_withdrawal: 0u8.into(),
        }),
    );

    // unstake and withdraw all tokens
    anyone.unstake().unwrap();
    let value: Option<Staker> = anyone.get_staker_info();
    assert_eq!(
        value,
        Some(Staker {
            memo:                      Some("memo".to_string().as_bytes().into()),
            tokens_staked:             0u8.into(),
            tokens_pending_withdrawal: 2u8.into(),
        }),
    );

    // unregister the data request executor by withdrawing all funds
    anyone.withdraw().unwrap();

    // fetching data request executor after unregistering should return None
    let value: Option<Staker> = anyone.get_staker_info();

    assert_eq!(value, None);
}

#[test]
fn executor_eligible() {
    let test_info = TestInfo::init();

    // someone registers a data request executor
    let anyone = test_info.new_executor("anyone", 40, 2);

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_executor_eligible = anyone.is_executor_eligible(dr_id);
    assert!(is_executor_eligible);
}

#[test]
fn multiple_executor_eligible() {
    let test_info = TestInfo::init();

    // someone registers a data request executor
    let val1 = test_info.new_executor("val1", 40, 2);
    let val2 = test_info.new_executor("val2", 20, 2);

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = val1
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_val1_executor_eligible = val1.is_executor_eligible(dr_id.to_string());
    let is_val2_executor_eligible = val2.is_executor_eligible(dr_id);

    if is_val1_executor_eligible && is_val2_executor_eligible {
        panic!("Both validators cannot be chosen at the same time");
    }

    assert!(is_val1_executor_eligible || is_val2_executor_eligible);
}

#[test]
fn multiple_executor_eligible_exact_replication_factor() {
    let test_info = TestInfo::init();

    // someone registers a data request executor
    let val1 = test_info.new_executor("val1", 40, 2);
    let val2 = test_info.new_executor("val2", 20, 2);

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = val1
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    // perform the check
    let is_val1_executor_eligible = val1.is_executor_eligible(dr_id.to_string());
    let is_val2_executor_eligible = val2.is_executor_eligible(dr_id);

    assert!(is_val1_executor_eligible && is_val2_executor_eligible);
}

#[test]
fn only_allow_active_stakers_to_be_eligible() {
    let test_info = TestInfo::init();

    let msg = msgs::ExecuteMsg::Staking(msgs::staking::execute::ExecuteMsg::SetStakingConfig(StakingConfig {
        minimum_stake:     Uint128::from(10u32),
        allowlist_enabled: false,
    }));

    test_info.execute::<()>(&test_info.creator(), &msg).unwrap();

    // someone registers a data request executor
    let val1 = test_info.new_executor("val1", 80, 21);
    let val2 = test_info.new_executor("val2", 20, 10);

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, 2);
    let dr_id = val1
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 2, None)
        .unwrap();

    // boh are staked and should be eligible
    let is_val1_executor_eligible = val1.is_executor_eligible(dr_id.clone());
    let is_val2_executor_eligible = val2.is_executor_eligible(dr_id);

    assert!(is_val1_executor_eligible);
    assert!(is_val2_executor_eligible);

    // val2 unstakes
    val2.unstake().unwrap();
    assert!(!val2.is_staker_executor());
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
    let test_info = TestInfo::init();
    let mut validators = Vec::with_capacity(VALIDATORS_AMOUNT);
    for validator_name in LARGE_SET_VALIDATOR_NAMES.iter() {
        let validator = test_info.new_executor(validator_name, 20, 2);
        validators.push(validator);
    }

    // someone registers a data request executor
    let anyone = test_info.new_account("anyone", 40);
    let replication_factor = 8;

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, replication_factor);
    let dr_id = anyone
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    let mut amount_eligible = 0;

    for validator in validators {
        let is_eligible = validator.is_executor_eligible(dr_id.to_string());

        if is_eligible {
            amount_eligible += 1;
        }
    }

    assert_eq!(amount_eligible, replication_factor);
}

#[test]
fn executor_not_eligible_if_dr_resolved() {
    let test_info = TestInfo::init();

    // someone registers a data request executor
    let anyone = test_info.new_account("anyone", 40);
    // Stake using TestInfo.stake method so we can set a memo
    anyone.stake_with_memo(2, "memo").unwrap();

    // post a data request
    let dr = data_requests::test_helpers::calculate_dr_id_and_args(1, 1);
    let dr_id = anyone
        .post_data_request(dr.clone(), vec![], vec![1, 2, 3], 1, None)
        .unwrap();

    let (_, proxy) = new_public_key();
    let reveal = RevealBody {
        dr_id:             dr_id.clone(),
        dr_block_height:   1,
        reveal:            "10".hash().into(),
        gas_used:          0,
        exit_code:         0,
        proxy_public_keys: vec![proxy.to_hex()],
    };
    let anyone_reveal_message = anyone.create_reveal_message(reveal);
    // commit
    anyone.commit_result(&dr_id, &anyone_reveal_message).unwrap();
    // reveal
    anyone.reveal_result(anyone_reveal_message).unwrap();

    // Owner removes the data request
    test_info
        .creator()
        .remove_data_request(
            dr_id.clone(),
            vec![DistributionMessage::DataProxyReward(DistributionDataProxyReward {
                payout_address: anyone.addr().to_string(),
                amount:         10u128.into(),
                public_key:     proxy.to_hex(),
            })],
        )
        .unwrap();

    // perform the check
    let is_executor_eligible = anyone.is_executor_eligible(dr_id);
    assert!(!is_executor_eligible);
}

#[test]
fn execute_messages_get_paused() {
    let test_info = TestInfo::init();

    // register a data request executor that can try to unstake after pausing
    let alice = test_info.new_executor("alice", 100, 10);

    // pause the contract
    test_info.creator().pause().unwrap();
    assert!(test_info.creator().is_paused());

    // try to have a new staker register
    let bob = test_info.new_account("bob", 100);
    let res = bob.stake(10);
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // try to have an existing staker unstake
    let res = alice.unstake();
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // try to have an existing staker withdraw rewards
    let res = alice.withdraw();
    assert!(res.is_err_and(|x| x.to_string().contains("pause")));

    // can still change the staking config
    let new_config = StakingConfig {
        minimum_stake:     10u8.into(),
        allowlist_enabled: false,
    };
    test_info.creator().set_staking_config(new_config).unwrap();
}

#[test]
fn staker_not_in_allowlist_withdrawing() {
    let test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake:     10u8.into(),
        allowlist_enabled: true,
    };
    test_info.creator().set_staking_config(new_config).unwrap();
    let alice = test_info.new_account("alice", 100);

    // add alice to the allowlist
    test_info.creator().add_to_allowlist(alice.pub_key()).unwrap();

    // now alice can register a data request executor
    alice.stake(10).unwrap();

    // remove alice from the allowlist
    test_info.creator().remove_from_allowlist(alice.pub_key()).unwrap();

    // alice withdraws all her funds removing her from the stakers map
    alice.withdraw().unwrap();
    assert!(alice.get_staker_info().is_none());
}

#[test]
fn minimum_stake_cannot_be_zero() {
    let test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake:     0u8.into(),
        allowlist_enabled: true,
    };
    let res = test_info.creator().set_staking_config(new_config);
    assert!(res.is_err_and(|x| x == ContractError::ZeroMinimumStakeToRegister));
}

#[test]
fn cannot_frontrun_unstake() {
    let test_info = TestInfo::init();

    // register a data request executor
    let alice = test_info.new_executor("alice", 1, 10);
    let fred = test_info.new_executor("fred", 1, 1);

    // alice produces the unstake message to unstake tokens
    let seq = alice.get_account_sequence();
    let factory = msgs::staking::execute::unstake::Execute::factory(
        alice.pub_key_hex(),
        test_info.chain_id(),
        test_info.contract_addr_str(),
        seq,
    );
    let proof = alice.prove(factory.get_hash());
    let msg = factory.create_message(proof);

    // fred frontruns alice's withdraw
    test_info.execute::<()>(&fred, &msg).unwrap();

    // verify alice tokens are in pending withdrawal
    let staker = alice.get_staker_info().unwrap();
    assert_eq!(staker.tokens_pending_withdrawal.u128(), 10);

    // fred should still have their same balance (1seda [original] - 1aseda [staked])
    let fred_expected_balance = seda_to_aseda(1.into()) - 1;
    let balance_fred = test_info.executor_balance("fred");
    assert_eq!(fred_expected_balance, balance_fred);

    // alice's balance should (1seda [original] - 10aseda [staked]
    let alice_expected_balance = seda_to_aseda(1.into()) - 10;
    let balance_alice = test_info.executor_balance("alice");
    assert_eq!(alice_expected_balance, balance_alice);
}

#[test]
fn cannot_frontrun_withdraw() {
    let test_info = TestInfo::init();

    // register a data request executor
    let alice = test_info.new_executor("alice", 1, 10);
    let fred = test_info.new_executor("fred", 1, 1);

    // alice unstakes all tokens
    alice.unstake().unwrap();

    // verify alice has 10 tokens pending withdrawal
    let staker = alice.get_staker_info().unwrap();
    assert_eq!(staker.tokens_pending_withdrawal.u128(), 10);

    // alice produces the withdraw message to withdraw their tokens
    let seq = alice.get_account_sequence();
    let factory = msgs::staking::execute::withdraw::Execute::factory(
        alice.pub_key_hex(),
        alice.addr().to_string(),
        test_info.chain_id(),
        test_info.contract_addr_str(),
        seq,
    );
    let proof = alice.prove(factory.get_hash());
    let msg = factory.create_message(proof);

    // fred frontruns alice's withdraw - but she should get the tokens not fred
    test_info.execute::<()>(&fred, &msg).unwrap();

    // fred should still have their same balance (1seda [original] - 1aseda [staked])
    let fred_expected_balance = seda_to_aseda(1.into()) - 1;
    let balance_fred = test_info.executor_balance("fred");
    assert_eq!(fred_expected_balance, balance_fred);

    // alice's balance  should be (1seda [original] - 10aseda [staked] + 10aseda [withdrawn]) = 1seda
    let alice_expected_balance = seda_to_aseda(1.into());
    let balance_alice = test_info.executor_balance("alice");
    assert_eq!(alice_expected_balance, balance_alice);
}

#[test]
#[should_panic(expected = "InvalidAddress")]
fn withdraw_to_invalid_address() {
    let test_info = TestInfo::init();

    let alice = test_info.new_executor("alice", 100u128, 10);
    alice.unstake().unwrap();

    alice.withdraw_to("not-an-address".to_string()).unwrap();
}

#[test]
fn query_paginated_executors_works() {
    let test_info = TestInfo::init();
    let alice_memo = "foo";
    let alice_memo_staker_value = Some(alice_memo.as_bytes().into());
    let alice = test_info.new_executor_with_memo("alice", 10, 1, alice_memo);

    // Alice is the only executor
    let response = alice.query_executors(0, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 1);
    assert_eq!(executors[0].memo, alice_memo_staker_value);
    assert_eq!(executors[0].public_key, alice.pub_key_hex());

    // Add more executors
    let bob_memo = "bar";
    let bob_memo_staker_value = Some(bob_memo.as_bytes().into());
    let charlie_memo = "baz";
    let charlie_memo_staker_value = Some(charlie_memo.as_bytes().into());
    let bob = test_info.new_executor_with_memo("bob", 10, 1, bob_memo);
    let charlie = test_info.new_executor_with_memo("charlie", 10, 1, charlie_memo);

    // Check Alice is still the first executor
    // check the others in order
    let response = alice.query_executors(0, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 3);
    assert_eq!(executors[0].memo, alice_memo_staker_value);
    assert_eq!(executors[1].memo, bob_memo_staker_value);
    assert_eq!(executors[2].memo, charlie_memo_staker_value);
    assert_eq!(executors[0].public_key, alice.pub_key_hex());
    assert_eq!(executors[1].public_key, bob.pub_key_hex());
    assert_eq!(executors[2].public_key, charlie.pub_key_hex());

    // Check limit works
    let response = alice.query_executors(0, 1);
    let executors = response.executors;
    assert_eq!(executors.len(), 1);
    assert_eq!(executors[0].memo, alice_memo_staker_value);
    assert_eq!(executors[0].public_key, alice.pub_key_hex());

    // Check offset works
    let response = alice.query_executors(1, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 2);
    assert_eq!(executors[0].memo, bob_memo_staker_value);
    assert_eq!(executors[1].memo, charlie_memo_staker_value);
    assert_eq!(executors[0].public_key, bob.pub_key_hex());
    assert_eq!(executors[1].public_key, charlie.pub_key_hex());
}

#[test]
fn query_paginated_executors_unstaking_all_funds() {
    let test_info = TestInfo::init();
    let alice = test_info.new_executor("alice", 10, 1);

    let response = alice.query_executors(0, 10);
    assert_eq!(response.executors.len(), 1);

    // Unstaking doesn't remove them from the StakersMap
    alice.unstake().unwrap();

    let response = alice.query_executors(0, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 1);
    assert_eq!(executors[0].tokens_staked, Uint128::new(0));
}

#[test]
fn query_paginated_executors_removed_from_allowlist() {
    let test_info = TestInfo::init();

    // enable allowlist
    let new_config = StakingConfig {
        minimum_stake:     1u8.into(),
        allowlist_enabled: true,
    };
    test_info.creator().set_staking_config(new_config).unwrap();

    let alice = test_info.new_account("alice", 10);
    test_info.creator().add_to_allowlist(alice.pub_key()).unwrap();
    alice.stake(1).unwrap();

    // Alice is the only executor
    let response = alice.query_executors(0, 10);
    assert_eq!(response.executors.len(), 1);

    // Remove from allowlist
    // They are still in the StakersMap till they withdraw
    test_info.creator().remove_from_allowlist(alice.pub_key()).unwrap();
    let response = alice.query_executors(0, 10);
    assert_eq!(response.executors.len(), 1);

    // Withdrawing all funds removes them from the StakersMap
    alice.withdraw().unwrap();
    let response = alice.query_executors(0, 10);
    assert_eq!(response.executors.len(), 0);
}

#[test]
fn query_paginated_executors_swap_removal() {
    let test_info = TestInfo::init();

    let new_config = StakingConfig {
        minimum_stake:     1u8.into(),
        allowlist_enabled: true,
    };
    test_info.creator().set_staking_config(new_config).unwrap();

    let alice = test_info.new_account("alice", 10);
    test_info.creator().add_to_allowlist(alice.pub_key()).unwrap();
    let bob = test_info.new_account("bob", 10);
    test_info.creator().add_to_allowlist(bob.pub_key()).unwrap();
    let charlie = test_info.new_account("charlie", 10);
    test_info.creator().add_to_allowlist(charlie.pub_key()).unwrap();

    let alice_memo = "foo";
    let alice_memo_staker_value = Some(alice_memo.as_bytes().into());
    let bob_memo = "bar";
    let bob_memo_staker_value = Some(bob_memo.as_bytes().into());
    let charlie_memo = "baz";
    let charlie_memo_staker_value = Some(charlie_memo.as_bytes().into());

    alice.stake_with_memo(1, alice_memo).unwrap();
    bob.stake_with_memo(1, bob_memo).unwrap();
    charlie.stake_with_memo(1, charlie_memo).unwrap();

    let response = alice.query_executors(0, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 3);
    assert_eq!(executors[0].memo, alice_memo_staker_value);
    assert_eq!(executors[1].memo, bob_memo_staker_value);
    assert_eq!(executors[2].memo, charlie_memo_staker_value);

    test_info.creator().remove_from_allowlist(alice.pub_key()).unwrap();
    alice.withdraw().unwrap();

    let response = alice.query_executors(0, 10);
    let executors = response.executors;
    assert_eq!(executors.len(), 2);
    assert_eq!(executors[0].memo, charlie_memo_staker_value);
    assert_eq!(executors[1].memo, bob_memo_staker_value);
}
