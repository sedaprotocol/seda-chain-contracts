use crate::tests::utils::{calculate_dr_id_and_args, get_dr_id, proper_instantiate, USER};
use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse, QuerySeedResponse};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use data_requests::utils::string_to_hash;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn post_data_request() {
    let (mut app, proxy_contract) = proper_instantiate();

    let (_, posted_dr) = calculate_dr_id_and_args(1, 3);
    // post the data request
    let msg = ProxyExecuteMsg::PostDataRequest { posted_dr };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app
        .execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    // try posting again, expecting error
    assert!(app.execute(Addr::unchecked(USER), cosmos_msg).is_err());

    // should be able to fetch data request
    let dr_id = get_dr_id(res);

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
        dr_id: string_to_hash("non-existent"),
    };
    let res: GetDataRequestResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value.is_none());
}

#[test]
fn get_seed() {
    let (app, proxy_contract) = proper_instantiate();

    let req = ProxyQueryMsg::QuerySeedRequest {};
    let res: QuerySeedResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &req)
        .unwrap();

    println!("block_height {}", res.block_height);
    println!("seed {}", res.seed);

    assert!(!res.seed.is_empty());
}
