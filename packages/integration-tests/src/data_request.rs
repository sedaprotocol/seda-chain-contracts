use crate::tests::utils::{proper_instantiate, USER};
use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse, PostDataRequestArgs};
use common::types::{Bytes, Hash, Memo};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use data_requests::state::DataRequestInputs;
use data_requests::utils::{hash_data_request, hash_update};
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};
use sha3::{Digest, Keccak256};

#[test]
fn post_data_request() {
    let (mut app, proxy_contract) = proper_instantiate();

    // format inputs to post data request
    let dr_binary_id: Hash = "".to_string();
    let tally_binary_id: Hash = "".to_string();
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();
    let replication_factor: u16 = 3;
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;
    let seda_payload: Bytes = Vec::new();
    let chain_id = 31337;
    let nonce = 1;
    let value = "test".to_string();
    let mut hasher = Keccak256::new();
    hash_update(&mut hasher, &chain_id);
    hash_update(&mut hasher, &nonce);
    hasher.update(value);
    let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
    let memo1: Memo = binary_hash.clone().into_bytes();
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
    let constructed_dr_id: String = hash_data_request(dr_inputs1);
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

    // try posting again, expecting error
    assert!(app.execute(Addr::unchecked(USER), cosmos_msg).is_err());

    // should be able to fetch data request
    // TODO: this is ugly to loop through events, use Response.data once it's merged
    let dr_id = &res.events.last().unwrap().attributes.last().unwrap().value;
    let msg = ProxyQueryMsg::GetDataRequest {
        dr_id: dr_id.clone(),
    };
    let res: GetDataRequestResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value.is_some());

    // can also fetch it via `get_data_requests_from_pool`
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: None,
        limit: None,
    };
    let res: GetDataRequestsFromPoolResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert_eq!(res.value.len(), 1);
    assert_eq!(res.value.first().unwrap().dr_id, dr_id.clone());

    // non-existent data request should fail
    let msg = ProxyQueryMsg::GetDataRequest {
        dr_id: "non-existent".to_string(),
    };
    let res: GetDataRequestResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value.is_none());
}
