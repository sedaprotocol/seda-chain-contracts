use cosmwasm_schema::write_api;

use common::msg::InstantiateMsg;
use common::msg::{StakingExecuteMsg, StakingQueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: StakingExecuteMsg,
        query: StakingQueryMsg,
    }
}
