use crate::tests::utils::{proper_instantiate, send_tokens, EXECUTOR_1, USER};

use common::{
    error::ContractError, msg::GetDataRequestExecutorResponse, state::DataRequestExecutor,
};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

use common::consts::INITIAL_MINIMUM_STAKE_TO_REGISTER;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn register_data_request_executor() {
    let (mut app, proxy_contract, _) = proper_instantiate();

    // send tokens from USER to executor1 so it can register
    send_tokens(&mut app, USER, EXECUTOR_1, 1);

    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();
    // register executor
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: GetDataRequestExecutorResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();

    assert_eq!(
        res,
        GetDataRequestExecutorResponse {
            value: Some(DataRequestExecutor {
                memo: Some("address".to_string()),
                tokens_staked: 1,
                tokens_pending_withdrawal: 0
            })
        }
    );
}

#[test]
fn unregister_data_request_executor() {
    let (mut app, proxy_contract, _) = proper_instantiate();

    // send tokens from USER to executor1 so it can register
    send_tokens(&mut app, USER, EXECUTOR_1, 1);

    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();
    // register executor
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: Addr::unchecked(EXECUTOR_1),
    };
    let res: GetDataRequestExecutorResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();

    assert_eq!(
        res,
        GetDataRequestExecutorResponse {
            value: Some(DataRequestExecutor {
                memo: Some("address".to_string()),
                tokens_staked: 1,
                tokens_pending_withdrawal: 0
            })
        }
    );

    // can't unregister the data request executor if it has staked tokens
    let msg = ProxyExecuteMsg::UnregisterDataRequestExecutor {};
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    // unregister executor
    let res = app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone());
    assert_eq!(
        res.is_err_and(|x| {
            x.source().unwrap().source().unwrap().to_string()
                == ContractError::ExecutorHasTokens.to_string()
        }),
        true
    );

    // unstake
    let msg = ProxyExecuteMsg::Unstake { amount: 1 };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();

    // withdraw
    let msg = ProxyExecuteMsg::Withdraw { amount: 1 };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();

    // unregister
    let msg = ProxyExecuteMsg::UnregisterDataRequestExecutor {};
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    // unregister executor
    app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg.clone())
        .unwrap();

    // fetching data request executor for an address that doesn't exist should return None
    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: Addr::unchecked("anyone"),
    };
    let res: GetDataRequestExecutorResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();

    assert_eq!(res, GetDataRequestExecutorResponse { value: None });
}
