#[cfg(not(feature = "cosmwasm"))]
use base64::Engine;

#[cfg(not(feature = "cosmwasm"))]
use crate::crypto::get_vrf;

#[cfg(not(feature = "cosmwasm"))]
const SIGNING_KEY: [u8; 32] = [
    219, 250, 64, 32, 30, 234, 99, 114, 97, 170, 110, 172, 152, 165, 220, 129, 127, 165, 104, 32, 6, 97, 222, 68, 164,
    143, 185, 62, 132, 40, 237, 146,
];
const PUBLIC_KEY: [u8; 33] = [
    3, 248, 41, 12, 151, 21, 115, 241, 156, 228, 228, 226, 57, 172, 191, 44, 41, 132, 80, 177, 87, 88, 64, 180, 49, 82,
    228, 233, 77, 87, 251, 171, 251,
];

#[test]
#[cfg(not(feature = "cosmwasm"))]
fn test_stake_execute_factory_create_message() {
    let pub_key_hex = hex::encode(PUBLIC_KEY);

    let chain_id = "test_chain_id";
    let contract_addr = "test_contract_addr";
    let sequence = 1u128.into();

    let memo = base64::prelude::BASE64_STANDARD.encode("memo".as_bytes());

    let factory =
        super::execute::stake::Execute::factory(pub_key_hex, Some(memo.clone()), chain_id, contract_addr, sequence)
            .unwrap();
    let proof = get_vrf().prove(&SIGNING_KEY, factory.get_hash()).unwrap();
    let msg = factory.create_message(proof);
    let mut msg_json = serde_json::to_value(&msg).unwrap();
    serde_json::to_writer(
        std::fs::File::create("stake_execute_factory_create_message.test.json").unwrap(),
        &msg_json["stake"].take(),
    )
    .unwrap();
}

#[test]
#[cfg(feature = "cosmwasm")]
fn test_stake_execute_verify() {
    let chain_id = "test_chain_id";
    let contract_addr = "test_contract_addr";
    let sequence = 1u128.into();

    let msg: super::execute::stake::Execute =
        serde_json::from_reader(std::fs::File::open("stake_execute_factory_create_message.test.json").unwrap())
            .unwrap();

    let memo = Some(cosmwasm_std::Binary::from("memo".as_bytes()));
    assert_eq!(msg.memo, memo);

    let public_key = hex::decode(&msg.public_key).unwrap();
    assert_eq!(public_key, PUBLIC_KEY);

    msg.verify(&public_key, chain_id, contract_addr, sequence).unwrap();
}
