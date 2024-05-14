use common::{error::ContractError, state::StakingConfig, test_utils::TestExecutor};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
    Addr,
};

use super::helpers::*;
use crate::test::helpers::{
    helper_accept_ownership,
    helper_get_owner,
    helper_get_pending_owner,
    helper_set_staking_config,
    helper_transfer_ownership,
    instantiate_staking_contract,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &coins(1000, "token"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate_staking_contract(deps.as_mut(), info).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn two_step_transfer_ownership() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(1000, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    let owner = helper_get_owner(deps.as_mut()).value;
    assert_eq!(owner, "owner");

    let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, None);

    // new-owner accepts ownership before owner calls transfer_ownership
    let info = mock_info("new-owner", &coins(2, "token"));

    let res = helper_accept_ownership(deps.as_mut(), info);
    assert!(res.is_err_and(|x| x == ContractError::NoPendingOwnerFound),);

    // non-owner initiates transfering ownership
    let info = mock_info("non-owner", &coins(2, "token"));

    let res = helper_transfer_ownership(deps.as_mut(), info, "new_owner".to_string());
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));

    // owner initiates transfering ownership
    let info = mock_info("owner", &coins(2, "token"));

    let res = helper_transfer_ownership(deps.as_mut(), info, "new_owner".to_string());
    assert!(res.is_ok());

    let owner = helper_get_owner(deps.as_mut()).value;
    assert_eq!(owner, "owner");

    let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, Some(Addr::unchecked("new_owner")));

    // non-owner accepts ownership
    let info = mock_info("non-owner", &coins(2, "token"));

    let res = helper_accept_ownership(deps.as_mut(), info);
    assert!(res.is_err_and(|x| x == ContractError::NotPendingOwner));

    // new owner accepts ownership
    let info = mock_info("new_owner", &coins(2, "token"));

    let res = helper_accept_ownership(deps.as_mut(), info);
    assert!(res.is_ok());

    let owner = helper_get_owner(deps.as_mut()).value;
    assert_eq!(owner, Addr::unchecked("new_owner"));

    let pending_owner = helper_get_pending_owner(deps.as_mut()).value;
    assert_eq!(pending_owner, None);
}

#[test]
fn set_staking_config() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(1000, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info.clone()).unwrap();

    let new_config = StakingConfig {
        minimum_stake_to_register:               100,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled:                       false,
    };

    // non-owner sets staking config
    let info = mock_info("non-owner", &coins(0, "token"));

    let res = helper_set_staking_config(deps.as_mut(), info, new_config.clone());
    assert!(res.is_err_and(|x| x == ContractError::NotOwner));

    // owner sets staking config
    let info = mock_info("owner", &coins(0, "token"));

    let res = helper_set_staking_config(deps.as_mut(), info, new_config);
    assert!(res.is_ok());
}

#[test]
pub fn allowlist_works() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    // update the config with allowlist enabled
    let info = mock_info("owner", &coins(0, "token"));
    let new_config = StakingConfig {
        minimum_stake_to_register:               100,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled:                       true,
    };
    let res = helper_set_staking_config(deps.as_mut(), info, new_config);
    assert!(res.is_ok());

    // alice tries to register a data request executor, but she's not on the allowlist
    let info = mock_info("alice", &coins(100, "token"));
    let alice = TestExecutor::new("alice");
    let res = helper_register_executor(deps.as_mut(), info, &alice, None);
    dbg!(&res);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // add alice to the allowlist
    let info = mock_info("owner", &coins(0, "token"));
    let res = helper_add_to_allowlist(deps.as_mut(), info, alice.public_key.clone());
    assert!(res.is_ok());

    // now alice can register a data request executor
    let info = mock_info("alice", &coins(100, "token"));
    let res = helper_register_executor(deps.as_mut(), info, &alice, None);
    assert!(res.is_ok());

    // alice unstakes, withdraws, then unregisters herself
    let info = mock_info("alice", &coins(0, "token"));
    let _res = helper_unstake(deps.as_mut(), info.clone(), &alice, 100).unwrap();
    let info = mock_info("alice", &coins(0, "token"));
    let _res = helper_withdraw(deps.as_mut(), info.clone(), &alice, 100).unwrap();
    let info = mock_info("alice", &coins(0, "token"));
    let res = helper_unregister_executor(deps.as_mut(), info, &alice);
    assert!(res.is_ok());

    // remove alice from the allowlist
    let info = mock_info("owner", &coins(0, "token"));
    let res = helper_remove_from_allowlist(deps.as_mut(), info, alice.public_key.clone());
    assert!(res.is_ok());

    // now alice can't register a data request executor
    let info = mock_info("alice", &coins(2, "token"));
    let res = helper_register_executor(deps.as_mut(), info, &alice, None);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // update the config to disable the allowlist
    let info = mock_info("owner", &coins(0, "token"));
    let new_config = StakingConfig {
        minimum_stake_to_register:               100,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled:                       false,
    };
    let res = helper_set_staking_config(deps.as_mut(), info, new_config);
    assert!(res.is_ok());

    // now alice can register a data request executor
    let info = mock_info("alice", &coins(100, "token"));
    let res = helper_register_executor(deps.as_mut(), info, &alice, None);
    assert!(res.is_ok());
}
