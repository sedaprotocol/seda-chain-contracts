use common::msg::{InstantiateMsg, StakingExecuteMsg, StakingQueryMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: StakingExecuteMsg,
        query: StakingQueryMsg,
    }
}
