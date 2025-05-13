use cosmwasm_std::Uint128;
use seda_common::{msgs::staking::StakingConfig, types::ToHexStr};

use crate::{error::ContractError, TestInfo};

#[test]
fn get_owner() {
    let test_info = TestInfo::init();
    let someone = test_info.new_account("someone", 2);

    let owner_addr = someone.get_owner();
    assert_eq!(owner_addr, test_info.creator().addr());
}

#[test]
fn pending_owner_no_transfer() {
    let test_info = TestInfo::init();
    let someone = test_info.new_account("someone", 2);

    let pending_owner = someone.get_pending_owner();
    assert_eq!(pending_owner, None);
}

#[test]
fn non_owner_cannot_transfer_ownership() {
    let test_info = TestInfo::init();

    // non-owner cannot transfer ownership
    let non_owner = test_info.new_account("non-owner", 2);
    let res = non_owner.transfer_ownership(&non_owner);
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));
}

#[test]
fn two_step_transfer_ownership() {
    let test_info = TestInfo::init();

    // new-owner cannot accept ownership without a transfer
    let new_owner = test_info.new_account("new-owner", 2);
    let res = new_owner.accept_ownership();
    assert!(res.is_err_and(|x| x == ContractError::NoPendingOwnerFound),);

    // owner initiates transfering ownership
    let res = test_info.creator().transfer_ownership(&new_owner);
    assert!(res.is_ok());

    // owner is still the owner
    let owner_addr = new_owner.get_owner();
    assert_eq!(owner_addr, test_info.creator().addr());

    // new owner is pending owner
    let pending_owner = new_owner.get_pending_owner();
    assert_eq!(pending_owner, Some(new_owner.addr()));

    // new owner accepts ownership
    let res = new_owner.accept_ownership();
    assert!(res.is_ok());

    // new owner is now the owner
    let owner = new_owner.get_owner();
    assert_eq!(owner, new_owner.addr());

    // pending owner is now None
    let pending_owner = new_owner.get_pending_owner();
    assert_eq!(pending_owner, None);
}

#[test]
fn non_transferee_cannont_accept_ownership() {
    let test_info = TestInfo::init();

    // new-owner cannot accept ownership without a transfer
    let new_owner = test_info.new_account("new-owner", 2);

    // owner initiates transfering ownership
    test_info.creator().transfer_ownership(&new_owner).unwrap();

    // non-owner accepts ownership
    let non_owner = test_info.new_account("non-owner", 2);
    let res = non_owner.accept_ownership();
    assert!(res.is_err_and(|x| x == ContractError::NotPendingOwner));
}

#[test]
fn allowlist_works() {
    let test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake:     10u8.into(),
        allowlist_enabled: true,
    };
    test_info.creator().set_staking_config(new_config).unwrap();

    // alice tries to register a data request executor, but she's not on the allowlist
    let alice = test_info.new_account("alice", 100);
    let res = alice.stake(10);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // add alice to the allowlist
    test_info.creator().add_to_allowlist(alice.pub_key()).unwrap();

    // now alice can register a data request executor
    alice.stake(10).unwrap();

    // alice unstakes, withdraws, then unregisters herself
    alice.unstake().unwrap();
    alice.withdraw().unwrap();
    // test_info.unregister(&alice).unwrap();

    // remove alice from the allowlist
    test_info.creator().remove_from_allowlist(alice.pub_key()).unwrap();

    // now alice can't register a data request executor
    let res = alice.stake(2);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // update the config to disable the allowlist
    let new_config = StakingConfig {
        minimum_stake:     10u8.into(),
        allowlist_enabled: false,
    };
    test_info.creator().set_staking_config(new_config).unwrap();

    // now alice can register a data request executor
    alice.stake(100).unwrap();
}

#[test]
fn allowlist_query_works() {
    let test_info = TestInfo::init();

    // add alice to the allowlist
    let alice = test_info.new_account("alice", 100);
    test_info.creator().add_to_allowlist(alice.pub_key()).unwrap();

    // query the allowlist
    let allowlist = test_info.creator().get_allowlist();
    assert_eq!(allowlist.len(), 1);
    assert_eq!(allowlist[0], alice.pub_key().to_hex());

    // add bob to the allowlist
    let bob = test_info.new_account("bob", 100);
    test_info.creator().add_to_allowlist(bob.pub_key()).unwrap();

    // query the allowlist
    let allowlist = test_info.creator().get_allowlist();
    assert_eq!(allowlist.len(), 2);
    assert!(allowlist.contains(&alice.pub_key().to_hex()));
    assert!(allowlist.contains(&bob.pub_key().to_hex()));
}

#[test]
fn pause_works() {
    let test_info = TestInfo::init();
    let someone = test_info.new_account("someone", 2);

    // check that the contract is not paused
    assert!(!someone.is_paused());

    // pause the contract
    test_info.creator().pause().unwrap();
    assert!(someone.is_paused());

    // double pause leaves errors
    let err = test_info.creator().pause().unwrap_err();
    assert!(err.to_string().contains("Contract paused"));

    // check that sudo messages are not paused
    assert!(someone
        .remove_data_request("Doesn't matter".to_string(), vec![])
        .is_ok());

    // execute messages are paused
    let alice = test_info.new_account("alice", 100);
    assert!(alice.stake(10).is_err());

    // unpause the contract
    test_info.creator().unpause().unwrap();
    assert!(!someone.is_paused());

    // double unpause leaves errors
    let err = test_info.creator().unpause().unwrap_err();
    assert!(err.to_string().contains("Contract not paused"));
}

#[test]
fn removing_from_allowlist_unstakes() {
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

    // alice should no longer have any stake bu only tokens to withdraw
    let staker = alice.get_staker_info().unwrap();
    assert_eq!(staker.tokens_staked, Uint128::new(0));
    assert_eq!(staker.tokens_pending_withdrawal, Uint128::new(10));
}
