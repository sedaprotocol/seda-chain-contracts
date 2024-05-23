use cosmwasm_schema::write_api;
use seda_contract::{
    msg::InstantiateMsg,
    msgs::{ExecuteMsg, QueryMsg},
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
