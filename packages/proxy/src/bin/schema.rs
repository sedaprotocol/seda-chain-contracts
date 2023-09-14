use cosmwasm_schema::write_api;

use proxy_contract::msg::{InstantiateMsg, ProxyExecuteMsg, ProxyQueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ProxyExecuteMsg,
        query: ProxyQueryMsg,
    }
}
