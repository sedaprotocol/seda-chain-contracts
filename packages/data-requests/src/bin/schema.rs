use cosmwasm_schema::write_api;

use common::msg::{DataRequestsExecuteMsg, DataRequestsQueryMsg};
use data_requests::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: DataRequestsExecuteMsg,
        query: DataRequestsQueryMsg,
    }
}
