use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
    Addr,
};

use super::{test_helpers, TestExecutor};
use crate::{error::ContractError, msgs::staking::StakingConfig};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let creator = mock_info("creator", &coins(1000, "token"));

    // we can just call .unwrap() to assert this was a success
    let res = test_helpers::instantiate_staking_contract(deps.as_mut(), creator).unwrap();
    assert_eq!(0, res.messages.len());
}
