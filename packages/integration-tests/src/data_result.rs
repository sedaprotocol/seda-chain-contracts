use crate::tests::utils::{
    calculate_commitment, get_dr_id, helper_commit_result, helper_post_dr, helper_reveal_result,
    proper_instantiate, send_tokens, EXECUTOR_1, EXECUTOR_2, USER,
};
use common::msg::{
    GetCommittedDataResultResponse, GetResolvedDataResultResponse, GetRevealedDataResultResponse,
};
use common::state::Reveal;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use data_requests::helpers::calculate_dr_id_and_args;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};
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

    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        "nonexistent".to_string(),
        "result".to_string(),
        Addr::unchecked(EXECUTOR_1),
    );
    assert!(res.is_err());

    let (_, posted_dr) = calculate_dr_id_and_args(1, 2);

    let res = helper_post_dr(
        &mut app,
        proxy_contract.clone(),
        posted_dr,
        Addr::unchecked(USER),
    )
    .unwrap();

    // get dr_id
    let dr_id = get_dr_id(res);

    let commitment1 = calculate_commitment("2000", EXECUTOR_1);

    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment1,
        Addr::unchecked(EXECUTOR_1),
    )
    .unwrap();

    let commitment2 = calculate_commitment("3000", EXECUTOR_2);

    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment2,
        Addr::unchecked(EXECUTOR_2),
    )
    .unwrap();

    // should be able to fetch committed data result
    let msg = ProxyQueryMsg::GetCommittedDataResult {
        dr_id: dr_id.to_string(),
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

    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        reveal1,
        Addr::unchecked(EXECUTOR_1),
    )
    .unwrap();

    // should be able to fetch revealed data result
    let msg = ProxyQueryMsg::GetRevealedDataResult {
        dr_id: dr_id.to_string(),
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

    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        reveal2,
        Addr::unchecked(EXECUTOR_2),
    )
    .unwrap();

    // now data request is resolved, let's check
    let msg = ProxyQueryMsg::GetResolvedDataResult {
        dr_id: dr_id.to_string(),
    };
    let res: GetResolvedDataResultResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert_eq!(res.value.dr_id, dr_id.to_string());
}

#[test]
fn ineligible_post_data_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    let (_, posted_dr) = calculate_dr_id_and_args(1, 2);

    let res = helper_post_dr(
        &mut app,
        proxy_contract.clone(),
        posted_dr,
        Addr::unchecked(USER),
    )
    .unwrap();

    // get dr_id
    let dr_id = get_dr_id(res);

    let commitment1 = calculate_commitment("2000", EXECUTOR_1);

    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment1,
        Addr::unchecked(EXECUTOR_1),
    );

    assert!(res.is_err());
}
