use seda_common::msgs::staking::StakingConfig;

use crate::{error::ContractError, TestInfo};

#[test]
fn get_owner() {
    let test_info = TestInfo::init();

    let owner_addr = test_info.get_owner();
    assert_eq!(owner_addr, test_info.creator().addr());
}

#[test]
fn pending_owner_no_transfer() {
    let test_info = TestInfo::init();

    let pending_owner = test_info.get_pending_owner();
    assert_eq!(pending_owner, None);
}

#[test]
fn non_owner_cannot_transfer_ownership() {
    let mut test_info = TestInfo::init();

    // non-owner cannot transfer ownership
    let non_owner = test_info.new_executor("non-owner", Some(2));
    let res = test_info.transfer_ownership(&non_owner, &non_owner);
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));
}

#[test]
fn two_step_transfer_ownership() {
    let mut test_info = TestInfo::init();

    // new-owner cannot accept ownership without a transfer
    let new_owner = test_info.new_executor("new-owner", Some(2));
    let res = test_info.accept_ownership(&new_owner);
    assert!(res.is_err_and(|x| x == ContractError::NoPendingOwnerFound),);

    // owner initiates transfering ownership
    let res = test_info.transfer_ownership(&test_info.creator(), &new_owner);
    assert!(res.is_ok());

    // owner is still the owner
    let owner_addr = test_info.get_owner();
    assert_eq!(owner_addr, test_info.creator().addr());

    // new owner is pending owner
    let pending_owner = test_info.get_pending_owner();
    assert_eq!(pending_owner, Some(new_owner.addr()));

    // new owner accepts ownership
    let res = test_info.accept_ownership(&new_owner);
    assert!(res.is_ok());

    // new owner is now the owner
    let owner = test_info.get_owner();
    assert_eq!(owner, new_owner.addr());

    // pending owner is now None
    let pending_owner = test_info.get_pending_owner();
    assert_eq!(pending_owner, None);
}

#[test]
fn non_transferee_cannont_accept_ownership() {
    let mut test_info = TestInfo::init();

    // new-owner cannot accept ownership without a transfer
    let new_owner = test_info.new_executor("new-owner", Some(2));

    // owner initiates transfering ownership
    test_info.transfer_ownership(&test_info.creator(), &new_owner).unwrap();

    // non-owner accepts ownership
    let non_owner = test_info.new_executor("non-owner", Some(2));
    let res = test_info.accept_ownership(&non_owner);
    assert!(res.is_err_and(|x| x == ContractError::NotPendingOwner));
}

#[test]
pub fn allowlist_works() {
    let mut test_info = TestInfo::init();

    // update the config with allowlist enabled
    let new_config = StakingConfig {
        minimum_stake_to_register:               10u8.into(),
        minimum_stake_for_committee_eligibility: 20u8.into(),
        allowlist_enabled:                       true,
    };
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();

    // alice tries to register a data request executor, but she's not on the allowlist
    let mut alice = test_info.new_executor("alice", Some(100));
    let res = test_info.stake(&mut alice, None, 10);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // add alice to the allowlist
    test_info
        .add_to_allowlist(&test_info.creator(), alice.pub_key())
        .unwrap();

    // now alice can register a data request executor
    test_info.stake(&mut alice, None, 10).unwrap();

    // alice unstakes, withdraws, then unregisters herself
    test_info.unstake(&alice, 10).unwrap();
    test_info.withdraw(&mut alice, 10).unwrap();
    // test_info.unregister(&alice).unwrap();

    // remove alice from the allowlist
    test_info
        .remove_from_allowlist(&test_info.creator(), alice.pub_key())
        .unwrap();

    // now alice can't register a data request executor
    let res = test_info.stake(&mut alice, None, 2);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // update the config to disable the allowlist
    let new_config = StakingConfig {
        minimum_stake_to_register:               10u8.into(),
        minimum_stake_for_committee_eligibility: 20u8.into(),
        allowlist_enabled:                       false,
    };
    test_info.set_staking_config(&test_info.creator(), new_config).unwrap();

    // now alice can register a data request executor
    test_info.stake(&mut alice, None, 100).unwrap();
}
