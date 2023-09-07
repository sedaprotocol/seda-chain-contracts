use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Response};

#[test]
fn store_and_read_binary() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &coins(2, "token"));

    // Instantiate the contract
    instantiate(deps.as_mut(), mock_env(), info.clone(), InstantiateMsg {}).unwrap();
}
