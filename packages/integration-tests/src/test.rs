use crate::helpers::CwTemplateContract;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

pub fn seda_chain_contracts_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        seda_chain_contracts::contract::execute,
        seda_chain_contracts::contract::instantiate,
        seda_chain_contracts::contract::query,
    );
    Box::new(contract)
}

pub fn wasm_bin_storage_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        wasm_bin_storage::contract::execute,
        wasm_bin_storage::contract::instantiate,
        wasm_bin_storage::contract::query,
    );
    Box::new(contract)
}

#[allow(dead_code)]
const USER: &str = "USER";
const ADMIN: &str = "ADMIN";
const NATIVE_DENOM: &str = "denom";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
    })
}

fn proper_instantiate() -> (App, CwTemplateContract, CwTemplateContract) {
    let mut app = mock_app();

    // instantiate wasm-bin-storage
    let wasm_bin_storage_template_id = app.store_code(wasm_bin_storage_template());
    let msg = wasm_bin_storage::msg::InstantiateMsg {};
    let wasm_bin_storage_template_contract_addr = app
        .instantiate_contract(
            wasm_bin_storage_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();
    let wasm_bin_storage_template_contract =
        CwTemplateContract(wasm_bin_storage_template_contract_addr);

    // instantiate seda-chain-contracts
    let seda_chain_contracts_template_id = app.store_code(seda_chain_contracts_template());
    let msg = seda_chain_contracts::msg::InstantiateMsg {
        token: "token".to_string(),
        wasm_storage_contract_address: wasm_bin_storage_template_contract.addr(),
    };
    let seda_chain_contracts_template_contract_addr = app
        .instantiate_contract(
            seda_chain_contracts_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();
    let seda_chain_contracts_template_contract =
        CwTemplateContract(seda_chain_contracts_template_contract_addr);

    (
        app,
        wasm_bin_storage_template_contract,
        seda_chain_contracts_template_contract,
    )
}

#[test]
fn post_data_request() {
    let (mut app, _wasm_bin_storage_template_contract, seda_chain_contracts_template_contract) =
        proper_instantiate();

    let msg = seda_chain_contracts::msg::ExecuteMsg::PostDataRequest {
        dr_id: "0x899e5a9b45bf4a4ffb24c5b11b8cbcd7808182addd3c3ac21ee0a5d321e7ff81".to_string(), // expected
        nonce: 1,
        chain_id: 31337,
        value: "test".to_string(),
    };
    let cosmos_msg = seda_chain_contracts_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
}
