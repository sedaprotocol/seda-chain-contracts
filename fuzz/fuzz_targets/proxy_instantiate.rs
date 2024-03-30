#![no_main]

use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use libfuzzer_sys::fuzz_target;
use proxy_contract::msg::ProxyInstantiateMsg;

const ADMIN: &str = "admin";
pub const NATIVE_DENOM: &str = "seda";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        // TODO fuzz number of users?
        // unsure if this is necessary for this test
        let users = vec![("user1", 100u128)];

        for (user, amount) in users {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(user),
                    vec![Coin {
                        denom: "NATIVE_DENOM".to_string(),
                        amount: Uint128::new(amount),
                    }],
                )
                .unwrap();
        }
    })
}

fn proxy_contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        proxy_contract::contract::execute,
        proxy_contract::contract::instantiate,
        proxy_contract::contract::query,
    )
    .with_sudo(proxy_contract::contract::sudo)
    .with_reply(proxy_contract::contract::reply);
    Box::new(contract)
}

fuzz_target!(|msg: ProxyInstantiateMsg| {
    // fuzzed code goes here
    let mut app = mock_app();

    // instantiate proxy-contract
    let proxy_contract_code_id = app.store_code(proxy_contract_template());
    app.instantiate_contract(
        proxy_contract_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap();
});
