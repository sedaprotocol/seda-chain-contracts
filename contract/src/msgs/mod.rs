use cosmwasm_std::*;
use cw_storage_plus::{Item, Map};
use seda_contract_common::msgs::{
    self,
    staking::{Staker, StakingConfig},
};

use crate::{
    contract::CONTRACT_VERSION,
    crypto::{hash, verify_proof},
    error::ContractError,
    types::*,
};

pub mod data_requests;
pub mod owner;
pub mod staking;

pub trait QueryHandler {
    fn query(self, deps: Deps, env: Env) -> StdResult<Binary>;
}

pub trait ExecuteHandler {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError>;
}

impl ExecuteHandler for msgs::ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            msgs::ExecuteMsg::DataRequest(msg) => msg.execute(deps, env, info),
            msgs::ExecuteMsg::Staking(msg) => msg.execute(deps, env, info),
            msgs::ExecuteMsg::Owner(msg) => msg.execute(deps, env, info),
        }
    }
}

impl QueryHandler for msgs::QueryMsg {
    fn query(self, deps: Deps, env: Env) -> StdResult<Binary> {
        match self {
            msgs::QueryMsg::DataRequest(msg) => msg.query(deps, env),
            msgs::QueryMsg::Staking(msg) => msg.query(deps, env),
            msgs::QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}
