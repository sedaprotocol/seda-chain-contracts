use crate::tests::utils::{
    get_dr_id, proper_instantiate, send_tokens, EXECUTOR_1, EXECUTOR_2, USER,
};
use common::msg::{
    GetCommittedDataResultResponse, GetResolvedDataResultResponse, GetRevealedDataResultResponse,
    PostDataRequestArgs,
};
use common::state::Reveal;
use common::types::{Bytes, Hash};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use data_requests::state::DataRequestInputs;
use data_requests::utils::hash_data_request;
use data_requests::utils::string_to_hash;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};
use sha3::{Digest, Keccak256};
use staking::consts::MINIMUM_STAKE_TO_REGISTER;
#[test]
fn commit_reveal_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    // send tokens from USER to executor1 and executor2 so they can register
    send_tokens(&mut app, USER, EXECUTOR_1, 1);
    send_tokens(&mut app, USER, EXECUTOR_2, 1);
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        p2p_multi_address: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, MINIMUM_STAKE_TO_REGISTER)
        .unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();
    app.execute(Addr::unchecked(EXECUTOR_2), cosmos_msg)
        .unwrap();

    // can't commit on a data request if it doesn't exist
    let msg = ProxyExecuteMsg::CommitDataResult {
        dr_id: string_to_hash("nonexistent".to_owned()),
        commitment: string_to_hash("result".to_owned()),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone());
    assert!(res.is_err());

    // format inputs to post data request with replication factor of 2
    let dr_binary_id: Hash = string_to_hash("dr_binary_id".to_owned());
    let tally_binary_id: Hash = string_to_hash("tally_binary_id".to_owned());
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();
    let replication_factor: u16 = 2;
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;
    let seda_payload: Bytes = Vec::new();
    let chain_id: u128 = 31337;
    let nonce: u128 = 1;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo1 = hasher.finalize().to_vec();
    let payback_address: Bytes = Vec::new();
    let dr_inputs1 = DataRequestInputs {
        dr_binary_id: dr_binary_id.clone(),
        tally_binary_id: tally_binary_id.clone(),
        dr_inputs: dr_inputs.clone(),
        tally_inputs: tally_inputs.clone(),
        memo: memo1.clone(),
        replication_factor,
        gas_price,
        gas_limit,
        seda_payload: seda_payload.clone(),
        payback_address: payback_address.clone(),
    };
    let constructed_dr_id: Hash = hash_data_request(dr_inputs1);
    let payback_address: Bytes = Vec::new();
    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        dr_id: constructed_dr_id,
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo: memo1,
        replication_factor,
        gas_price,
        gas_limit,
        seda_payload,
        payback_address,
    };

    // post the data request
    let msg = ProxyExecuteMsg::PostDataRequest { posted_dr };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app
        .execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    // get dr_id
    let dr_id = get_dr_id(res).clone();

    // executor1 commits on the data request
    let reveal = "2000";
    let salt = EXECUTOR_1;
    let mut hasher = Keccak256::new();
    hasher.update(reveal.as_bytes());
    hasher.update(salt.as_bytes());
    let digest = hasher.finalize();
    let commitment1 = format!("0x{}", hex::encode(digest));
    let msg = ProxyExecuteMsg::CommitDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        commitment: string_to_hash(commitment1),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg)
        .unwrap();

    // executor2 commits on the data request
    let reveal = "3000";
    let salt = EXECUTOR_2;
    let mut hasher = Keccak256::new();
    hasher.update(reveal.as_bytes());
    hasher.update(salt.as_bytes());
    let digest = hasher.finalize();
    let commitment2 = format!("0x{}", hex::encode(digest));
    let msg = ProxyExecuteMsg::CommitDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        commitment: string_to_hash(commitment2),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_2), cosmos_msg)
        .unwrap();

    // should be able to fetch committed data result
    let msg = ProxyQueryMsg::GetCommittedDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: GetCommittedDataResultResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value.is_some());
    println!("res: {:?}", res);

    // executor1 reveals data result
    let reveal1 = Reveal {
        reveal: "2000".to_string(),
        salt: "executor1".to_string(),
    };
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        reveal: reveal1,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg)
        .unwrap();

    // should be able to fetch revealed data result
    let msg = ProxyQueryMsg::GetRevealedDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: GetRevealedDataResultResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value.is_some());

    // executor 2 reveals data result
    let reveal2 = Reveal {
        reveal: "3000".to_string(),
        salt: "executor2".to_string(),
    };
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id: string_to_hash(dr_id.clone()),
        reveal: reveal2,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_2), cosmos_msg)
        .unwrap();

    // now data request is resolved, let's check
    let msg = ProxyQueryMsg::GetResolvedDataResult {
        dr_id: string_to_hash(dr_id.clone()),
    };
    let res: GetResolvedDataResultResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let dr_id_bytes: [u8; 32] = string_to_hash(dr_id.clone());
    assert_eq!(res.value.dr_id, dr_id_bytes);
}

#[test]
fn ineligible_post_data_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    // post a data request
    let dr_binary_id: Hash = string_to_hash("dr_binary_id".to_owned());
    let tally_binary_id: Hash = string_to_hash("tally_binary_id".to_owned());
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();
    let replication_factor: u16 = 2;
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;
    let seda_payload: Bytes = Vec::new();
    let chain_id: u128 = 31337;
    let nonce: u128 = 1;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo1 = hasher.finalize().to_vec();
    let payback_address: Bytes = Vec::new();
    let dr_inputs1 = DataRequestInputs {
        dr_binary_id: dr_binary_id.clone(),
        tally_binary_id: tally_binary_id.clone(),
        dr_inputs: dr_inputs.clone(),
        tally_inputs: tally_inputs.clone(),
        memo: memo1.clone(),
        replication_factor,
        gas_price,
        gas_limit,
        seda_payload: seda_payload.clone(),
        payback_address: payback_address.clone(),
    };
    let constructed_dr_id: Hash = hash_data_request(dr_inputs1);
    let payback_address: Bytes = Vec::new();
    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        dr_id: constructed_dr_id,
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo: memo1,
        replication_factor,
        gas_price,
        gas_limit,
        seda_payload,
        payback_address,
    };
    let msg = ProxyExecuteMsg::PostDataRequest { posted_dr };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app
        .execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    // get dr_id
    let dr_id = get_dr_id(res);

    // ineligible shouldn't be able to post a data result
    let reveal = "2000";
    let salt = EXECUTOR_1;
    let mut hasher = Keccak256::new();
    hasher.update(reveal.as_bytes());
    hasher.update(salt.as_bytes());
    let digest = hasher.finalize();
    let commitment1 = format!("0x{}", hex::encode(digest));
    let msg = ProxyExecuteMsg::CommitDataResult {
        dr_id: string_to_hash(dr_id),
        commitment: string_to_hash(commitment1),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg);
    assert!(res.is_err());
}
