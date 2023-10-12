use crate::tests::utils::calculate_dr_id_and_args;
use crate::tests::utils::{
    calculate_commitment, get_dr_id, helper_commit_result, helper_post_dr, helper_reveal_result,
    proper_instantiate, send_tokens, EXECUTOR_1, EXECUTOR_2, EXECUTOR_3, USER,
};
use common::consts::INITIAL_MINIMUM_STAKE_TO_REGISTER;
use common::error::ContractError;
use common::msg::{
    GetCommittedDataResultResponse, GetResolvedDataResultResponse, GetRevealedDataResultResponse,
    IsDataRequestExecutorEligibleResponse,
};
use common::state::Reveal;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn commit_reveal_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    // executor 1 should be ineligible to register
    let msg = ProxyQueryMsg::IsDataRequestExecutorEligible {
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: IsDataRequestExecutorEligibleResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value == false);

    // send tokens from USER to executor1, executor2, executor3 so they can register
    send_tokens(&mut app, USER, EXECUTOR_1, 1);
    send_tokens(&mut app, USER, EXECUTOR_2, 1);
    send_tokens(&mut app, USER, EXECUTOR_3, 1);
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        p2p_multi_address: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();
    app.execute(Addr::unchecked(EXECUTOR_2), cosmos_msg.clone())
        .unwrap();
    app.execute(Addr::unchecked(EXECUTOR_3), cosmos_msg)
        .unwrap();

    // executor 1 should be eligible to register
    let msg = ProxyQueryMsg::IsDataRequestExecutorEligible {
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: IsDataRequestExecutorEligibleResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    assert!(res.value == true);

    // can't post data result on nonexistent data request
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

    // executor 1 commits
    let commitment1 = calculate_commitment("2000", EXECUTOR_1);
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment1,
        Addr::unchecked(EXECUTOR_1),
    )
    .unwrap();

    // can't reveal until replication factor is reached
    let reveal1 = Reveal {
        reveal: "2000".to_string(),
        salt: "executor1".to_string(),
    };
    let res = helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        reveal1.clone(),
        Addr::unchecked(EXECUTOR_1),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::RevealNotStarted)
    );

    // executor 2 commits
    let commitment2 = calculate_commitment("3000", EXECUTOR_2);
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment2.clone(),
        Addr::unchecked(EXECUTOR_2),
    )
    .unwrap();

    // can't commit on the data request a second time
    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        commitment2,
        Addr::unchecked(EXECUTOR_2),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::AlreadyCommitted)
    );

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

    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        reveal1.clone(),
        Addr::unchecked(EXECUTOR_1),
    )
    .unwrap();

    // can't reveal on the data request a second time
    let res = helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id.to_string(),
        reveal1,
        Addr::unchecked(EXECUTOR_1),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::AlreadyRevealed)
    );

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

    // executor 3 can't reveal since no commit was posted
    let reveal3 = Reveal {
        reveal: "4000".to_string(),
        salt: "executor3".to_string(),
    };
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id: dr_id.to_string(),
        reveal: reveal3.clone(),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(EXECUTOR_3), cosmos_msg.clone());
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::NotCommitted)
    );

    // reveal must match commitment
    let reveal2 = Reveal {
        reveal: "9999".to_string(),
        salt: "executor2".to_string(),
    };
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id: dr_id.to_string(),
        reveal: reveal2,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(EXECUTOR_2), cosmos_msg);
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::RevealMismatch)
    );

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

    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::IneligibleExecutor)
    );
}
