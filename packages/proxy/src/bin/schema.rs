use cosmwasm_schema::write_api;

use proxy_contract::msg::{ProxyExecuteMsg, ProxyInstantiateMsg, ProxyQueryMsg, ProxySudoMsg};

fn main() {
    write_api! {
        instantiate: ProxyInstantiateMsg,
        execute: ProxyExecuteMsg,
        query: ProxyQueryMsg,
        sudo: ProxySudoMsg,
    }
}
