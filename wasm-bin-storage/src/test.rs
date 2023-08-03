// TODO more tests
// TODO separate file for tests
use crate::contract::{store_binary, read_binary};
use cosmwasm_std::testing::{mock_dependencies,  mock_info};
use cosmwasm_std::{coins, Binary, Response};

#[test]
fn store_and_read_binary() {
    let mut deps = mock_dependencies();

    let key = "myKey".to_string();
    let data = Binary::from("myData".as_bytes());
    let description = "my data binary".to_string();

    let info = mock_info("sender", &coins(2, "token"));

    // Call the StoreBinary handler
    let res = store_binary(deps.as_mut(), info.clone(), &key, data.clone(), description.clone()).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("method", "store_binary")
            .add_attribute("new_binary_key", key.clone())
    );

    // Test that a bin with the same name would fail.
    let duplicate = store_binary(deps.as_mut(), info, &key, data.clone(), description.clone());
    assert!(duplicate.is_err());

    // Now query the data back
    let res = read_binary(&deps.as_mut(), &key).unwrap();
    assert_eq!(res.binary, data);
    assert_eq!(res.description, description);
}
