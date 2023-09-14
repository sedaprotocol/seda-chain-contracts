use cosmwasm_schema::write_api;

use common::msg::{StakingExecuteMsg, StakingQueryMsg};
use staking::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: StakingExecuteMsg,
        query: StakingQueryMsg,
    }
}
