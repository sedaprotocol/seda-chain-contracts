use crate::tests::utils::{proper_instantiate, EXECUTOR_1};
use common::error::ContractError;
use common::msg::GetContractResponse;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg, ProxySudoMsg};

#[test]
fn sudo_set_contract_address() {
    let (mut app, proxy_contract, _) = proper_instantiate();

    // query initial contract address
    let msg = ProxyQueryMsg::GetDataRequestsContract {};
    let res: GetContractResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let initial_contract_address = res.value;

    // expect error when non-owner tries to set contract address via Execute call after the initial set
    let msg = ProxyExecuteMsg::SetDataRequests {
        contract: "some_address".to_string(),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    let res = app.execute(Addr::unchecked(EXECUTOR_1), cosmos_msg);
    assert_eq!(
        res.unwrap_err().downcast_ref::<ContractError>(),
        Some(&ContractError::NotContractCreator)
    );

    // only sudo can change contract address
    let msg = ProxySudoMsg::SetDataRequests {
        contract: "new_contract_address".to_string(),
    };
    let cosmos_msg = proxy_contract.sudo(msg);
    let _res = app.sudo(cosmos_msg);

    // query new contract address
    let msg = ProxyQueryMsg::GetDataRequestsContract {};
    let res: GetContractResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let new_contract_address = res.value;

    assert_ne!(initial_contract_address, new_contract_address);
    assert_eq!(new_contract_address, "new_contract_address");
}
