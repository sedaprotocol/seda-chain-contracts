use crate::tests::utils::{proper_instantiate, CwTemplateContract, EXECUTOR_1};
use common::msg::GetContractResponse;
use common::state::StakingConfig;
use common::{error::ContractError, msg::GetStakingConfigResponse};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use proxy_contract::msg::{ProxyExecuteMsg, ProxyQueryMsg, ProxySudoMsg};
use staking::msg::StakingSudoMsg;

#[test]
fn sudo_set_contract_address() {
    let (mut app, proxy_contract) = proper_instantiate();

    // query initial contract address
    let msg = ProxyQueryMsg::GetDataRequestsContract {};
    let res: GetContractResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let initial_contract_address = res.value;

    // expect error when non-admin tries to set contract address via Execute call after the initial set
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

#[test]
fn sudo_set_staking_config() {
    let (mut app, proxy_contract) = proper_instantiate();

    // query initial config
    let msg = ProxyQueryMsg::GetStakingConfig;
    let res: GetStakingConfigResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let initial_config = res.value;

    // query staking contract address on proxy
    let msg = ProxyQueryMsg::GetStakingContract{};
    let res: GetContractResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let staking_contract_address = res.value;
    let staking_contract = CwTemplateContract(Addr::unchecked(staking_contract_address));

    // set new config
    let new_config = StakingConfig {
        minimum_stake_to_register: 100,
        minimum_stake_for_committee_eligibility: 200,
    };
    let msg = StakingSudoMsg::SetStakingConfig {
        config: new_config.clone(),
    };
    let cosmos_msg = staking_contract.sudo_staking(msg);
    let _res = app.sudo(cosmos_msg);

    // assert new config is different
    let msg = ProxyQueryMsg::GetStakingConfig;
    let res: GetStakingConfigResponse = app
        .wrap()
        .query_wasm_smart(proxy_contract.addr(), &msg)
        .unwrap();
    let new_config = res.value;
    assert_ne!(initial_config, new_config);
}
