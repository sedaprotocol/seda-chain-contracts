use common::{error::ContractError, state::StakingConfig, test_utils::TestExecutor};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
};

use super::helpers::*;

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
    let res = helper_register_executor(deps.as_mut(), info, &alice, None, None);
    assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

    // add alice to the allowlist
    let info = mock_info("owner", &coins(0, "token"));
    let res = helper_add_to_allowlist(deps.as_mut(), info, "alice".to_string(), None);
    assert!(res.is_ok());

    // now alice can register a data request executor
    let info = mock_info("alice", &coins(100, "token"));
    let res = helper_register_executor(deps.as_mut(), info, &alice, None, None);
    assert!(res.is_ok());

    // alice unstakes, withdraws, then unregisters herself
    let info = mock_info("alice", &coins(0, "token"));
    let _res = helper_unstake(deps.as_mut(), info.clone(), &alice, 100, None).unwrap();
    let info = mock_info("alice", &coins(0, "token"));
    let _res = helper_withdraw(deps.as_mut(), info.clone(), &alice, 100, None).unwrap();
    let info = mock_info("alice", &coins(0, "token"));
    let res = helper_unregister_executor(deps.as_mut(), info, &alice, None);
    assert!(res.is_ok());

    // remove alice from the allowlist
    let info = mock_info("owner", &coins(0, "token"));
    let res = helper_remove_from_allowlist(deps.as_mut(), info, "alice".to_string(), None);
    assert!(res.is_ok());

    // now alice can't register a data request executor
    let info = mock_info("alice", &coins(2, "token"));
    let res = helper_register_executor(deps.as_mut(), info, &alice, None, None);
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
    let res = helper_register_executor(deps.as_mut(), info, &alice, None, None);
    assert!(res.is_ok());
}
