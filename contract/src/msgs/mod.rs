use cosmwasm_std::*;
use cw_storage_plus::{Item, Map};
use seda_common::msgs::{
    self,
    staking::{Staker, StakingConfig},
    SudoMsg,
};

use crate::{common_types::*, contract::CONTRACT_VERSION, error::ContractError, types::*};

pub mod data_requests;
mod data_structures;
pub mod owner;
pub mod staking;

pub trait QueryHandler {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError>;
}

pub trait ExecuteHandler {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError>;
}

pub trait SudoHandler {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError>;
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
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        match self {
            msgs::QueryMsg::DataRequest(msg) => msg.query(deps, env),
            msgs::QueryMsg::Staking(msg) => msg.query(deps, env),
            msgs::QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}

impl SudoHandler for SudoMsg {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::DataRequest(sudo) => sudo.sudo(deps, env),
        }
    }
}
