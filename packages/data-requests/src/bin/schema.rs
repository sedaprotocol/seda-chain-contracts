use common::msg::{DataRequestsExecuteMsg, DataRequestsQueryMsg, InstantiateMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: DataRequestsExecuteMsg,
        query: DataRequestsQueryMsg,
    }
}
