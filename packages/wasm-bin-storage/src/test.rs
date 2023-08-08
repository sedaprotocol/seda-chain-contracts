use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::BinaryStruct;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Response};

const MB: usize = 1024 * 1024;

fn create_data(size_mb: usize) -> Vec<u8> {
    let size = size_mb * MB;
    let mut data = Vec::with_capacity(size);

    for i in 0..size {
        data.push((i % 256) as u8);
    }

    data
}

#[test]
fn store_and_read_binary() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &coins(2, "token"));

    // Instantiate the contract
    instantiate(deps.as_mut(), mock_env(), info.clone(), InstantiateMsg {}).unwrap();

    // Define the binary data
    let key = "0xe1bd58bbd853c00ee1d05a27c596e00f9bae26c87ca3276f2d46e45b24cfb4ff".to_string();
    let data = create_data(2);
    let description = "my data binary".to_string();

    // Call the StoreBinary handler
    let msg = ExecuteMsg::NewEntry {
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

    // Now query the data back
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryEntry { key }).unwrap();
    let value: BinaryStruct = from_binary(&res).unwrap();
    assert_eq!(value.binary, data);
    assert_eq!(value.description, description);
}
