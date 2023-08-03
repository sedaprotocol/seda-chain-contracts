// TODO more tests
// use crate::contract::{store_binary, query_binary};
use crate::contract::{execute, query};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::BinaryStruct;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Binary, Response};

#[test]
fn store_and_read_binary() {
    let mut deps = mock_dependencies();

    let key = "myKey".to_string();
    let data = Binary::from("myData".as_bytes());
    let description = "my data binary".to_string();

    let info = mock_info("sender", &coins(2, "token"));

    // Call the StoreBinary handler
    let msg = ExecuteMsg::NewEntry {
        key: key.clone(),
        binary: data.clone(),
        description: description.clone(),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("method", "store_binary")
            .add_attribute("new_binary_key", key.clone())
    );

    // Test that a bin with the same name would fail.
    let duplicate = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(duplicate.is_err());

    // Now query the data back
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryEntry { key }).unwrap();
    let value: BinaryStruct = from_binary(&res).unwrap();
    assert_eq!(value.binary, data);
    assert_eq!(value.description, description);
}
