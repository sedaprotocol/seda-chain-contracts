use crate::tests::utils::{proper_instantiate, OWNER, USER};

use common::{
    error::ContractError,
    msg::{GetDataRequestExecutorResponse, StakingExecuteMsg},
    state::{DataRequestExecutor, StakingConfig},
};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

use common::consts::INITIAL_MINIMUM_STAKE_TO_REGISTER;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg};

#[test]
fn allowlist_works() {
    let (mut app, proxy_contract, staking_contract) = proper_instantiate();

    // update the config with allowlist enabled
    let config = StakingConfig {
        minimum_stake_to_register: 1,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled: true,
    };
    let msg = StakingExecuteMsg::SetStakingConfig { config };
    let cosmos_msg = staking_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(OWNER), cosmos_msg.clone())
        .unwrap();

    // user tries to register a data request executor, but she's not on the allowlist
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();

    // registering executor should fail
    let res = app.execute(Addr::unchecked(USER), cosmos_msg.clone());

    assert_eq!(
        res.is_err_and(|x| {
            x.source().unwrap().source().unwrap().to_string()
                == ContractError::NotOnAllowlist.to_string()
        }),
        true
    );

    // add user to allowlist
    let msg = ProxyExecuteMsg::AddToAllowlist {
        address: Addr::unchecked(USER),
    };

    let cosmos_msg = proxy_contract.call(msg).unwrap();

    app.execute(Addr::unchecked(OWNER), cosmos_msg.clone())
        .unwrap();

    // user can now register
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();

    // register executor
    app.execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: Addr::unchecked(USER),
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

    // unstake
    let msg = ProxyExecuteMsg::Unstake { amount: 1 };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    // withdraw
    let msg = ProxyExecuteMsg::Withdraw { amount: 1 };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    // unregister
    let msg = ProxyExecuteMsg::UnregisterDataRequestExecutor {};
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    // unregister executor
    app.execute(Addr::unchecked(USER), cosmos_msg.clone())
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

    // rm user from allowlist
    let msg = ProxyExecuteMsg::RemoveFromAllowlist {
        address: Addr::unchecked(USER),
    };

    let cosmos_msg = proxy_contract.call(msg).unwrap();

    app.execute(Addr::unchecked(OWNER), cosmos_msg.clone())
        .unwrap();

    // registering executor should fail
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();

    let res = app.execute(Addr::unchecked(USER), cosmos_msg.clone());
    assert_eq!(
        res.is_err_and(|x| {
            x.source().unwrap().source().unwrap().to_string()
                == ContractError::NotOnAllowlist.to_string()
        }),
        true
    );

    // update the config to disable allowlist
    let config = StakingConfig {
        minimum_stake_to_register: 1,
        minimum_stake_for_committee_eligibility: 200,
        allowlist_enabled: false,
    };
    let msg = StakingExecuteMsg::SetStakingConfig { config };
    let cosmos_msg = staking_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(OWNER), cosmos_msg.clone())
        .unwrap();

    // user can now register
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor {
        memo: Some("address".to_string()),
    };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();

    // register executor
    app.execute(Addr::unchecked(USER), cosmos_msg.clone())
        .unwrap();

    let msg = ProxyQueryMsg::GetDataRequestExecutor {
        executor: Addr::unchecked(USER),
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
