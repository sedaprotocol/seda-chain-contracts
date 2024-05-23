use common::{
    error::ContractError,
    msg::{
        GetCommittedDataResultResponse,
        GetDataRequestsFromPoolResponse,
        GetResolvedDataResultResponse,
        GetRevealedDataResultResponse,
        IsDataRequestExecutorEligibleResponse,
    },
    state::RevealBody,
    test_utils::TestExecutor,
    types::SimpleHash,
};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

use crate::tests::utils::{
    calculate_dr_id_and_args,
    get_dr_id,
    helper_commit_result,
    helper_post_dr,
    helper_reg_dr_executor,
    helper_reveal_result,
    proper_instantiate,
    reveal_hash,
    send_tokens,
    USER,
};

#[test]
fn commit_reveal_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    let exec_1 = TestExecutor::new("exec_1");
    let exec_2 = TestExecutor::new("exec_2");
    let exec_3 = TestExecutor::new("exec_3");

    // executor 1 should be ineligible to register
    let msg = ProxyQueryMsg::IsDataRequestExecutorEligible {
        executor: exec_1.public_key.clone(),
    };
    let res: IsDataRequestExecutorEligibleResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    assert!(!res.value);

    // send tokens from USER to executor1, executor2, executor3 so they can register
    send_tokens(&mut app, USER, exec_1.name, 1);
    send_tokens(&mut app, USER, exec_2.name, 1);
    send_tokens(&mut app, USER, exec_3.name, 1);

    // register executors
    let memo = Some("address".to_string());
    helper_reg_dr_executor(&mut app, proxy_contract.clone(), &exec_1, memo.clone()).unwrap();
    helper_reg_dr_executor(&mut app, proxy_contract.clone(), &exec_2, memo.clone()).unwrap();
    helper_reg_dr_executor(&mut app, proxy_contract.clone(), &exec_3, memo.clone()).unwrap();

    // check if executors are eligible register
    // executor 1 should be eligible to register
    let elig_exec_1 = ProxyQueryMsg::IsDataRequestExecutorEligible {
        executor: exec_1.public_key.clone(),
    };
    let res: IsDataRequestExecutorEligibleResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &elig_exec_1)
        .unwrap();
    assert!(res.value);

    // can't post data result on nonexistent data request
    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_1,
        "nonexistent".simple_hash(),
        "result".simple_hash(),
        Addr::unchecked(exec_1.name),
    );
    assert!(res.is_err());

    let posted_dr = calculate_dr_id_and_args(1, 2);

    let res = helper_post_dr(&mut app, proxy_contract.clone(), posted_dr, Addr::unchecked(USER)).unwrap();

    // get dr_id
    let dr_id = get_dr_id(res);

    let reveal1 = RevealBody {
        reveal:    "2000".to_string().into_bytes(),
        salt:      exec_1.salt(),
        exit_code: 0,
        gas_used:  0,
    };
    let (commitment1, reveal1_sig_bytes) = reveal_hash(&reveal1, None);
    let reveal1_sig = exec_1.sign(reveal1_sig_bytes);

    // executor 1 commits
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_1,
        dr_id,
        commitment1,
        Addr::unchecked(exec_1.name),
    )
    .unwrap();

    // can't reveal until replication factor is reached
    let res = helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id,
        reveal1.clone(),
        reveal1_sig.clone(),
        Addr::unchecked(exec_1.name),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::RevealNotStarted)
    );

    // executor 2 commits
    let (commitment2, _) = reveal_hash(&reveal1, Some(exec_2.name));
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_2,
        dr_id,
        commitment2,
        Addr::unchecked(exec_2.name),
    )
    .unwrap();

    // can't commit on the data request a second time
    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_2,
        dr_id,
        commitment2,
        Addr::unchecked(exec_2.name),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::AlreadyCommitted)
    );

    // should be able to fetch committed data result
    let msg = ProxyQueryMsg::GetCommittedDataResult {
        dr_id,
        executor: exec_1.public_key.clone(),
    };
    let res: GetCommittedDataResultResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    assert!(res.value.is_some());

    // can't add another commitment since replication factor is reached
    let (commitment3, _) = reveal_hash(&reveal1, Some(exec_3.name));
    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_3,
        dr_id,
        commitment3,
        Addr::unchecked(exec_3.name),
    );

    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::RevealStarted)
    );

    // exeuctor 1 reveals data result
    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id,
        reveal1.clone(),
        reveal1_sig.clone(),
        Addr::unchecked(exec_1.name),
    )
    .unwrap();

    // can't reveal on the data request a second time
    let res = helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id,
        reveal1.clone(),
        reveal1_sig,
        Addr::unchecked(exec_1.name),
    );
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::AlreadyRevealed)
    );

    // should be able to fetch revealed data result
    let msg = ProxyQueryMsg::GetRevealedDataResult {
        dr_id,
        executor: exec_1.public_key.clone(),
    };
    let res: GetRevealedDataResultResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    assert!(res.value.is_some());

    // executor 3 can't reveal since no commit was posted
    let reveal3 = RevealBody {
        reveal:    "4000".to_string().into_bytes(),
        salt:      exec_3.salt(),
        exit_code: 0,
        gas_used:  0,
    };
    let (_, reveal3_sig_bytes) = reveal_hash(&reveal3, None);
    let reveal3_sig = exec_3.sign(reveal3_sig_bytes);
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id,
        reveal: reveal3.clone(),
        signature: reveal3_sig,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(exec_3.name), cosmos_msg.clone());
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::NotCommitted)
    );

    // reveal must match commitment
    let wrong_reveal = RevealBody {
        reveal:    "9999".to_string().into_bytes(),
        salt:      exec_2.salt(),
        exit_code: 0,
        gas_used:  0,
    };
    let (_, wrong_reveal_sig_bytes) = reveal_hash(&wrong_reveal, None);
    let wrong_reveal_sig = exec_2.sign(wrong_reveal_sig_bytes);
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id,
        reveal: wrong_reveal,
        signature: wrong_reveal_sig,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(exec_2.name), cosmos_msg);
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::RevealMismatch)
    );

    let reveal2 = RevealBody {
        reveal:    "2000".to_string().into_bytes(),
        salt:      exec_2.salt(),
        exit_code: 0,
        gas_used:  0,
    };
    let (_, reveal2_sig_bytes) = reveal_hash(&reveal2, None);
    let reveal2_sig = exec_2.sign(reveal2_sig_bytes);
    // executor 2 reveals data result
    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id,
        reveal2,
        reveal2_sig,
        Addr::unchecked(exec_2.name),
    )
    .unwrap();

    // now data request is resolved, let's check
    let msg = ProxyQueryMsg::GetResolvedDataResult { dr_id };
    let res: GetResolvedDataResultResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    assert_eq!(res.value.dr_id, dr_id);
}

#[test]
fn ineligible_post_data_result() {
    let (mut app, proxy_contract) = proper_instantiate();

    let posted_dr = calculate_dr_id_and_args(1, 2);

    let res = helper_post_dr(&mut app, proxy_contract.clone(), posted_dr, Addr::unchecked(USER)).unwrap();

    // get dr_id
    let dr_id = get_dr_id(res);

    let exec_1 = TestExecutor::new("exec_1");
    let reveal = RevealBody {
        reveal:    "2000".to_string().into_bytes(),
        salt:      exec_1.salt(),
        exit_code: 0,
        gas_used:  0,
    };

    let (commitment1, _) = reveal_hash(&reveal, None);

    let res = helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_1,
        dr_id,
        commitment1,
        Addr::unchecked(exec_1.name),
    );

    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::IneligibleExecutor)
    );
}

#[test]
fn pop_and_swap_in_pool() {
    let (mut app, proxy_contract) = proper_instantiate();

    let exec_1 = TestExecutor::new("exec_1");
    let exec_2 = TestExecutor::new("exec_2");

    // send tokens from USER to executor1 and executor2 so they can register
    send_tokens(&mut app, USER, exec_1.name, 1);
    send_tokens(&mut app, USER, exec_2.name, 1);

    // register executors
    let memo = Some("address".to_string());
    helper_reg_dr_executor(&mut app, proxy_contract.clone(), &exec_1, memo.clone()).unwrap();
    helper_reg_dr_executor(&mut app, proxy_contract.clone(), &exec_2, memo.clone()).unwrap();

    // post three drs

    let posted_dr = calculate_dr_id_and_args(1, 2);
    let res = helper_post_dr(&mut app, proxy_contract.clone(), posted_dr, Addr::unchecked(USER)).unwrap();
    let dr_id_1 = get_dr_id(res);
    let posted_dr = calculate_dr_id_and_args(2, 2);
    let res = helper_post_dr(&mut app, proxy_contract.clone(), posted_dr, Addr::unchecked(USER)).unwrap();
    let dr_id_2 = get_dr_id(res);
    let posted_dr = calculate_dr_id_and_args(3, 2);
    let res = helper_post_dr(&mut app, proxy_contract.clone(), posted_dr, Addr::unchecked(USER)).unwrap();
    let dr_id_3 = get_dr_id(res);

    // check dr 1, 2, 3 are in pool
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: None,
        limit:    None,
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 3);
    assert_eq!(fetched_drs[0].id, dr_id_1);
    assert_eq!(fetched_drs[1].id, dr_id_2);
    assert_eq!(fetched_drs[2].id, dr_id_3);

    // `GetDataRequestsFromPool` with position = 0 and limit = 1 should return dr 1
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: Some(0),
        limit:    Some(1),
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 1);
    assert_eq!(fetched_drs[0].id, dr_id_1);

    // resolve dr 1

    let reveal1 = RevealBody {
        reveal:    "2000".to_string().into_bytes(),
        salt:      exec_1.salt(),
        exit_code: 0,
        gas_used:  0,
    };

    // executor 1 commits
    let (commitment1, reveal1_sig_bytes) = reveal_hash(&reveal1, None);
    let reveal1_sig = exec_1.sign(reveal1_sig_bytes);
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_1,
        dr_id_1,
        commitment1,
        Addr::unchecked(exec_1.name),
    )
    .unwrap();

    let reveal2 = RevealBody {
        reveal:    "3000".to_string().into_bytes(),
        salt:      exec_2.salt(),
        exit_code: 0,
        gas_used:  0,
    };
    // executor 2 commits
    let (commitment2, reveal2_sig_bytes) = reveal_hash(&reveal2, None);
    let reveal2_sig = exec_2.sign(reveal2_sig_bytes);
    helper_commit_result(
        &mut app,
        proxy_contract.clone(),
        &exec_2,
        dr_id_1,
        commitment2,
        Addr::unchecked(exec_2.name),
    )
    .unwrap();

    // executor 1 reveals
    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id_1,
        reveal1.clone(),
        reveal1_sig.clone(),
        Addr::unchecked(exec_1.name),
    )
    .unwrap();

    // executor 2 reveals
    helper_reveal_result(
        &mut app,
        proxy_contract.clone(),
        dr_id_1,
        reveal2,
        reveal2_sig,
        Addr::unchecked(exec_2.name),
    )
    .unwrap();

    // pool is now of size two, the position of dr 2 and 3 should be swapped
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: None,
        limit:    None,
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 2);
    assert_eq!(fetched_drs[0].id, dr_id_3);
    assert_eq!(fetched_drs[1].id, dr_id_2);

    // `GetDataRequestsFromPool` with position = 1 should return dr 2
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: Some(1),
        limit:    None,
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 1);
    assert_eq!(fetched_drs[0].id, dr_id_2);

    // `GetDataRequestsFromPool` with limit = 1 should return dr 3
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: None,
        limit:    Some(1),
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 1);
    assert_eq!(fetched_drs[0].id, dr_id_3);

    // `GetDataRequestsFromPool` with position = 2 or 3 should return empty array
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: Some(2),
        limit:    None,
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 0);
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: Some(3),
        limit:    None,
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 0);

    // `GetDataRequestsFromPool` with limit = 0 should return empty array
    let msg = ProxyQueryMsg::GetDataRequestsFromPool {
        position: None,
        limit:    Some(0),
    };
    let res: GetDataRequestsFromPoolResponse = app.wrap().query_wasm_smart(proxy_contract.addr(), &msg).unwrap();
    let fetched_drs = res.value;
    assert_eq!(fetched_drs.len(), 0);
}
