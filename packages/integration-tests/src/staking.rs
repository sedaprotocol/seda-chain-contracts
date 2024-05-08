use crate::tests::utils::{
    helper_reg_dr_executor, proper_instantiate, send_tokens, NATIVE_DENOM, USER,
};

use common::test_utils::TestExecutor;
use common::{msg::GetDataRequestExecutorResponse, state::DataRequestExecutor};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

use cw_storage_plus::Endian;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn deposit_stake_withdraw() {
    let (mut app, proxy_contract) = proper_instantiate();

    let exec = TestExecutor::new("foo");

    // send tokens from USER to executor1 so it can register
    send_tokens(&mut app, USER, exec.name, 3);

    helper_reg_dr_executor(
        &mut app,
        proxy_contract.clone(),
        &exec,
        Some("address".to_string()),
    )
    .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: exec.public_key.clone(),
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
    let sender = Addr::unchecked(exec.name);
    let msg = ProxyExecuteMsg::DepositAndStake {
        signature: exec.sign([
            "deposit_and_stake".as_bytes().to_vec(),
            sender.as_bytes().to_vec(),
        ]),
    };
    let cosmos_msg = proxy_contract.call_with_deposit(msg, 2).unwrap();
    app.execute(sender.clone(), cosmos_msg.clone()).unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: exec.public_key.clone(),
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
    let amount = 2;
    let amount_bytes: [u8; 16] = amount.to_be_bytes();
    let msg = ProxyExecuteMsg::Unstake {
        signature: exec.sign([
            "unstake".as_bytes().to_vec(),
            amount_bytes.to_vec(),
            sender.as_bytes().to_vec(),
        ]),
        amount,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender.clone(), cosmos_msg.clone()).unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: exec.public_key.clone(),
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
        .query_balance(exec.name, NATIVE_DENOM)
        .unwrap()
        .amount
        .u128();
    assert_eq!(balance_before, 0);

    // withdraw 2
    let amount = 2;
    let amount_bytes: [u8; 16] = amount.to_be_bytes();
    let msg = ProxyExecuteMsg::Withdraw {
        signature: exec.sign([
            "withdraw".as_bytes().to_vec(),
            amount_bytes.to_vec(),
            sender.as_bytes().to_vec(),
        ]),
        amount,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(exec.name), cosmos_msg.clone())
        .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: exec.public_key.clone(),
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
        .query_balance(exec.name, NATIVE_DENOM)
        .unwrap()
        .amount
        .u128();
    assert_eq!(balance_after, 2);
}
