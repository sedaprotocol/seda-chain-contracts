use common::{error::ContractError, state::StakingConfig, test_utils::TestExecutor};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
    Addr,
};

use crate::test::helpers::{
    helper_accept_ownership,
    helper_get_owner,
    helper_get_pending_owner,
    helper_register_executor,
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
fn only_proxy_can_pass_caller() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(1000, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    // register a data request executor, while passing a sender
    let info = mock_info("anyone", &coins(2, "token"));
    let exec = TestExecutor::new("sender");

    let res = helper_register_executor(deps.as_mut(), info, &exec, None, Some("sender".to_string()));
    assert!(res.is_err_and(|x| x == ContractError::NotProxy));

    // register a data request executor from the proxy
    let info = mock_info("proxy", &coins(2, "token"));

    let _res = helper_register_executor(deps.as_mut(), info, &exec, None, Some("sender".to_string())).unwrap();
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
