use cosmwasm_schema::write_api;

use common::msg::{ExecuteMsg, QueryMsg};
use seda_chain_contracts::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
