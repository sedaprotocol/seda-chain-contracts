//! assert that the msg M serializes to the expected json J
//! if cosmwasm feature is enabled, also assert that we can deserialize the json back to the msg

#[track_caller]
pub fn assert_json_ser<M>(msg: M, expected_json: serde_json::Value)
where
    M: serde::Serialize + std::fmt::Debug + PartialEq,
{
    let msg_json = serde_json::to_value(&msg).unwrap();
    assert_eq!(msg_json, expected_json);
}

#[track_caller]
pub fn assert_json_deser<M>(msg: M, expected_json: serde_json::Value)
where
    M: serde::Serialize + std::fmt::Debug + PartialEq + serde::de::DeserializeOwned,
{
    let msg_json = serde_json::to_value(&msg).unwrap();
    assert_eq!(msg_json, expected_json);

    let msg_str = serde_json::to_string(&msg).unwrap();
    let deserialized: M = serde_json::from_str(&msg_str).unwrap();
    assert_eq!(msg, deserialized);
}
