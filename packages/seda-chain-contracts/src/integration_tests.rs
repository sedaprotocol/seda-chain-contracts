use crate::helpers::hash_update;
use crate::helpers::CwTemplateContract;
use crate::msg::ExecuteMsg;
use crate::msg::InstantiateMsg;

use crate::msg::PostDataRequestArgs;
use crate::types::Bytes;
use crate::types::Hash;
use crate::types::Memo;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use sha3::{Digest, Keccak256};

pub fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
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

fn proper_instantiate() -> (App, CwTemplateContract) {
    let mut app = mock_app();
    let cw_template_id = app.store_code(contract_template());

    let msg = InstantiateMsg {
        token: "token".to_string(),
    };
    let cw_template_contract_addr = app
        .instantiate_contract(
            cw_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

    (app, cw_template_contract)
}

#[test]
fn post_data_request() {
    let (mut app, cw_template_contract) = proper_instantiate();
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
    hash_update(&mut hasher, chain_id);
    hash_update(&mut hasher, nonce);
    hasher.update(value);
    let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
    let memo1: Memo = binary_hash.clone().into_bytes();
    let mut hasher = Keccak256::new();
    hasher.update(memo1.clone());

    let constructed_dr_id = format!("0x{}", hex::encode(hasher.finalize()));
    let payback_address: Bytes = Vec::new();
    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        dr_id: constructed_dr_id.clone(),

        dr_binary_id: dr_binary_id.clone(),
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
    let cosmos_msg = cw_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
}
