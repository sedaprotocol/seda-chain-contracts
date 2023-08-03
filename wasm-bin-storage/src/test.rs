use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::BinaryStruct;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Response};

#[test]
fn store_and_read_binary() {
    let mut deps = mock_dependencies();

    let info = mock_info("sender", &coins(2, "token"));

    // Instantiate the contract
    instantiate(deps.as_mut(), mock_env(), info.clone(), InstantiateMsg {}).unwrap();

    // Define the binary data
    let key = 0;
    let data = "myData".as_bytes();
    let description = "my data binary".to_string();

    // Call the StoreBinary handler
    let msg = ExecuteMsg::NewEntry {
        binary: data.to_vec(),
        description: description.clone(),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("method", "store_binary")
            .add_attribute("new_binary_key", key.clone().to_string())
    );

    // Now query the data back
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryEntry { key }).unwrap();
    let value: BinaryStruct = from_binary(&res).unwrap();
    assert_eq!(value.binary, data);
    assert_eq!(value.description, description);
}
