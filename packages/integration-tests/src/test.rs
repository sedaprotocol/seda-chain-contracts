use crate::helpers::CwTemplateContract;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use proxy_contract::msg::ExecuteMsg;
use seda_chain_contracts::msg::PostDataRequestArgs;
use seda_chain_contracts::state::DataRequestInputs;
use seda_chain_contracts::types::{Bytes, Hash, Memo};
use seda_chain_contracts::utils::{hash_data_request, hash_update};
use sha3::{Digest, Keccak256};

pub fn proxy_contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        proxy_contract::contract::execute,
        proxy_contract::contract::instantiate,
        proxy_contract::contract::query,
    );
    Box::new(contract)
}

pub fn seda_chain_contracts_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        seda_chain_contracts::contract::execute,
        seda_chain_contracts::contract::instantiate,
        seda_chain_contracts::contract::query,
    );
    Box::new(contract)
}

// pub fn wasm_bin_storage_template() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         wasm_bin_storage::contract::execute,
//         wasm_bin_storage::contract::instantiate,
//         wasm_bin_storage::contract::query,
//     );
//     Box::new(contract)
// }

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

fn proper_instantiate() -> (App, CwTemplateContract) {
    let mut app = mock_app();

    // instantiate wasm-bin-storage
    // let wasm_bin_storage_template_id = app.store_code(wasm_bin_storage_template());
    // let msg = wasm_bin_storage::msg::InstantiateMsg {};
    // let wasm_bin_storage_template_contract_addr = app
    //     .instantiate_contract(
    //         wasm_bin_storage_template_id,
    //         Addr::unchecked(ADMIN),
    //         &msg,
    //         &[],
    //         "test",
    //         None,
    //     )
    //     .unwrap();
    // let wasm_bin_storage_template_contract =
    //     CwTemplateContract(wasm_bin_storage_template_contract_addr);

    // instantiate proxy-contract
    let proxy_contract_template_id = app.store_code(proxy_contract_template());
    let msg = proxy_contract::msg::InstantiateMsg {};
    let proxy_contract_template_contract_addr = app
        .instantiate_contract(
            proxy_contract_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();
    let proxy_contract_template_contract =
        CwTemplateContract(proxy_contract_template_contract_addr);

    // instantiate seda-chain-contracts
    let seda_chain_contracts_template_id = app.store_code(seda_chain_contracts_template());
    let msg = seda_chain_contracts::msg::InstantiateMsg {
        token: "token".to_string(),
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
    // let seda_chain_contracts_template_contract =
    //     CwTemplateContract(seda_chain_contracts_template_contract_addr.clone());

    // set seda-chain-contract address on proxy-contract
    let msg = proxy_contract::msg::ExecuteMsg::SetSedaChainContracts {
        contract: seda_chain_contracts_template_contract_addr.to_string(),
    };
    let cosmos_msg = proxy_contract_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

    (app, proxy_contract_template_contract)
}

#[test]
fn post_data_request() {
    let (mut app, proxy_contract_template_contract) = proper_instantiate();
    let dr_binary_id: Hash = "".to_string();
    let tally_binary_id: Hash = "".to_string();
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();

    let replication_factor: u16 = 3;

    // set by dr creator
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;

    // set by relayer and SEDA protocol
    let seda_payload: Bytes = Vec::new();

    let chain_id = 31337;
    let nonce = 1;
    let value = "test".to_string();
    let mut hasher = Keccak256::new();
    hash_update(&mut hasher, &chain_id);
    hash_update(&mut hasher, &nonce);
    hasher.update(value);
    let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
    let memo1: Memo = binary_hash.clone().into_bytes();
    let payback_address: Bytes = Vec::new();

    let dr_inputs1 = DataRequestInputs {
        dr_binary_id: dr_binary_id.clone(),
        tally_binary_id: tally_binary_id.clone(),
        dr_inputs: dr_inputs.clone(),
        tally_inputs: tally_inputs.clone(),
        memo: memo1.clone(),
        replication_factor,

        gas_price,
        gas_limit,

        seda_payload: seda_payload.clone(),
        payback_address: payback_address.clone(),
    };
    let constructed_dr_id: String = hash_data_request(dr_inputs1);

    let payback_address: Bytes = Vec::new();
    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        dr_id: constructed_dr_id,

        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo: memo1,
        replication_factor,

        gas_price,
        gas_limit,

        seda_payload,
        payback_address,
    };
    let msg = ExecuteMsg::PostDataRequest { posted_dr };
    let cosmos_msg = proxy_contract_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
}
