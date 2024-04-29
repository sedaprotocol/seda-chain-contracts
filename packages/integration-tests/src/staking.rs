use crate::tests::utils::{proper_instantiate, send_tokens, EXECUTOR_1, NATIVE_DENOM, USER};

use common::{msg::GetDataRequestExecutorResponse, state::DataRequestExecutor};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

use common::consts::INITIAL_MINIMUM_STAKE_TO_REGISTER;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn deposit_stake_withdraw() {
    let (mut app, proxy_contract) = proper_instantiate();

    // send tokens from USER to executor1 so it can register
    send_tokens(&mut app, USER, EXECUTOR_1, 3);

    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
        public_key: vec![],
        signature: vec![],
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

    // deposit 2 more
    let msg = ProxyExecuteMsg::DepositAndStake {
        public_key: vec![],
        signature: vec![],
    };
    let cosmos_msg = proxy_contract.call_with_deposit(msg, 2).unwrap();
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
                tokens_staked: 3,
                tokens_pending_withdrawal: 0
            })
        }
    );

    // unstake 2
    let msg = ProxyExecuteMsg::Unstake {
        amount: 2,
        public_key: vec![],
        signature: vec![],
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
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
                tokens_pending_withdrawal: 2
            })
        }
    );

    let balance_before = app
        .wrap()
        .query_balance(EXECUTOR_1, NATIVE_DENOM)
        .unwrap()
        .amount
        .u128();
    assert_eq!(balance_before, 0);

    // withdraw 2
    let msg = ProxyExecuteMsg::Withdraw {
        amount: 2,
        public_key: vec![],
        signature: vec![],
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
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

    let balance_after = app
        .wrap()
        .query_balance(EXECUTOR_1, NATIVE_DENOM)
        .unwrap()
        .amount
        .u128();
    assert_eq!(balance_after, 2);
}
