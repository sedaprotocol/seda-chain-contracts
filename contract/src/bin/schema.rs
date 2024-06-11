use cosmwasm_schema::write_api;
use seda_common::msgs::*;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
