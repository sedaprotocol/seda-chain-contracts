use common::{error::ContractError, state::StakingConfig, test_utils::TestExecutor};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
    Addr,
};

use super::helpers;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let creator = mock_info("creator", &coins(1000, "token"));

    // we can just call .unwrap() to assert this was a success
    let res = helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn two_step_transfer_ownership() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(1000, "token"));
    let _res = helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();

    let owner_addr = helpers::get_owner(deps.as_mut()).value;
    assert_eq!(owner_addr, "owner");

    let pending_owner = helpers::get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, None);

    // new-owner accepts ownership before owner calls transfer_ownership
    let new_owner = mock_info("new-owner", &coins(2, "token"));
    let res = helpers::accept_ownership(deps.as_mut(), new_owner.clone());
    assert!(res.is_err_and(|x| x == ContractError::NoPendingOwnerFound),);

    // non-owner initiates transfering ownership
    let non_owner = mock_info("non-owner", &coins(2, "token"));
    let res = helpers::transfer_ownership(deps.as_mut(), non_owner.clone(), "new-owner".to_string());
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));

    // owner initiates transfering ownership
    let owner = mock_info("owner", &coins(2, "token"));
    let res = helpers::transfer_ownership(deps.as_mut(), owner, "new-owner".to_string());
    assert!(res.is_ok());

    let owner_addr = helpers::get_owner(deps.as_mut()).value;
    assert_eq!(owner_addr, "owner");

    let pending_owner = helpers::get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, Some(Addr::unchecked("new-owner")));

    // non-owner accepts ownership
    let res = helpers::accept_ownership(deps.as_mut(), non_owner);
    assert!(res.is_err_and(|x| x == ContractError::NotPendingOwner));

    // new owner accepts ownership
    let res = helpers::accept_ownership(deps.as_mut(), new_owner);
    assert!(res.is_ok());

    let owner = helpers::get_owner(deps.as_mut()).value;
    assert_eq!(owner, Addr::unchecked("new-owner"));

    let pending_owner = helpers::get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, None);
}

#[test]
fn set_staking_config() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(1000, "token"));
    let _res = helpers::instantiate_staking_contract(deps.as_mut(), creator.clone()).unwrap();

    let new_config = StakingConfig {
        minimum_stake_to_register:               200,
        minimum_stake_for_committee_eligibility: 100,
        allowlist_enabled:                       false,
    };

    // non-owner sets staking config
    let non_owner = mock_info("non-owner", &coins(0, "token"));
    let res = helpers::set_staking_config(deps.as_mut(), non_owner, new_config.clone());
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));

    // owner sets staking config
    let owner = mock_info("owner", &coins(0, "token"));
    let res = helpers::set_staking_config(deps.as_mut(), owner, new_config);
    assert!(res.is_ok());
}

#[test]
pub fn allowlist_works() {
    let mut deps = mock_dependencies();

    let creator = mock_info("creator", &coins(2, "token"));
    let _res = helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();

    // update the config with allowlist enabled
    let owner = mock_info("owner", &coins(0, "token"));
    let new_config = StakingConfig {
        minimum_stake_to_register:               100,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled:                       true,
    };
    let res = helpers::set_staking_config(deps.as_mut(), owner.clone(), new_config);
    assert!(res.is_ok());

    // alice tries to register a data request executor, but she's not on the allowlist
    let mut alice = TestExecutor::new("alice", Some(100));
    let res = helpers::reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // add alice to the allowlist
    let res = helpers::add_to_allowlist(deps.as_mut(), owner.clone(), alice.pub_key());
    assert!(res.is_ok());

    // now alice can register a data request executor
    let res = helpers::reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    assert!(res.is_ok());

    // alice unstakes, withdraws, then unregisters herself
    alice.set_amount(0);
    let _res = helpers::unstake(deps.as_mut(), alice.info(), &alice, 100).unwrap();
    let _res = helpers::withdraw(deps.as_mut(), alice.info(), &alice, 100).unwrap();
    let res = helpers::unregister(deps.as_mut(), alice.info(), &alice);
    assert!(res.is_ok());

    // remove alice from the allowlist
    let res = helpers::remove_from_allowlist(deps.as_mut(), owner.clone(), alice.pub_key());
    assert!(res.is_ok());

    // now alice can't register a data request executor
    alice.set_amount(2);
    let res = helpers::reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // update the config to disable the allowlist
    let new_config = StakingConfig {
        minimum_stake_to_register:               100,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled:                       false,
    };
    let res = helpers::set_staking_config(deps.as_mut(), owner, new_config);
    assert!(res.is_ok());

    // now alice can register a data request executor
    alice.set_amount(100);
    let res = helpers::reg_and_stake(deps.as_mut(), alice.info(), &alice, None);
    assert!(res.is_ok());
}
