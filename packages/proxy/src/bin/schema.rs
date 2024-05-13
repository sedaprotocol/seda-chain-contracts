use cosmwasm_schema::write_api;
use proxy_contract::msg::{InstantiateMsg, ProxyExecuteMsg, ProxyQueryMsg, ProxySudoMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ProxyExecuteMsg,
        query: ProxyQueryMsg,
        sudo: ProxySudoMsg,
    }
}
