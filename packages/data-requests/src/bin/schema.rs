use cosmwasm_schema::write_api;

use common::msg::InstantiateMsg;
use common::msg::{DataRequestsExecuteMsg, DataRequestsQueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: DataRequestsExecuteMsg,
        query: DataRequestsQueryMsg,
    }
}
