use super::staking::StakingConfig;
use crate::{error::ContractError, TestInfo};

#[test]
fn get_owner() {
    let test_info = TestInfo::init();

    let owner_addr = test_info.get_owner();
    assert_eq!(owner_addr.as_str(), test_info.creator().addr());
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
    let _test_info = TestInfo::init();

    // update the config with allowlist enabled
    let _new_config = StakingConfig {
        minimum_stake_to_register:               100u8.into(),
        minimum_stake_for_committee_eligibility: 200u8.into(),
        allowlist_enabled:                       true,
    };
    // let res = staking_test_info.set_staking_config(deps.as_mut(), owner.clone(), new_config);
    // assert!(res.is_ok());

    // // alice tries to register a data request executor, but she's not on the allowlist
    // let mut alice = TestExecutor::new("alice", Some(100));
    // let res = staking_test_info.reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    // assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // // add alice to the allowlist
    // let res = test_info.add_to_allowlist(deps.as_mut(), owner.clone(), alice.pub_key());
    // assert!(res.is_ok());

    // // now alice can register a data request executor
    // let res = staking_test_info.reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    // assert!(res.is_ok());

    // // alice unstakes, withdraws, then unregisters herself
    // alice.set_amount(0);
    // let _res = staking_test_info
    //     .unstake(deps.as_mut(), alice.info(), &alice, 100)
    //     .unwrap();
    // let _res = staking_test_info
    //     .withdraw(deps.as_mut(), alice.info(), &alice, 100)
    //     .unwrap();
    // let res = staking_test_info.unregister(deps.as_mut(), alice.info(), &alice);
    // assert!(res.is_ok());

    // // remove alice from the allowlist
    // let res = test_info.remove_from_allowlist(deps.as_mut(), owner.clone(), alice.pub_key());
    // assert!(res.is_ok());

    // // now alice can't register a data request executor
    // alice.set_amount(2);
    // let res = staking_test_info.reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    // assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // // update the config to disable the allowlist
    // let new_config = StakingConfig {
    //     minimum_stake_to_register:               100,
    //     minimum_stake_for_committee_eligibility: 200,
    //     allowlist_enabled:                       false,
    // };
    // let res = staking_test_info.set_staking_config(deps.as_mut(), owner, new_config);
    // assert!(res.is_ok());

    // // now alice can register a data request executor
    // alice.set_amount(100);
    // let res = staking_test_info.reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    // assert!(res.is_ok());
}
