use cosmwasm_std::*;
use cw_storage_plus::{Item, Map};
use seda_common::{
    crypto::*,
    msgs::{
        self,
        staking::{Staker, StakingConfig},
    },
};

use crate::{common_types::*, contract::CONTRACT_VERSION, error::ContractError, types::*};

pub mod data_requests;
pub mod owner;
pub mod staking;

pub trait QueryHandler {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError>;
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
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        match self {
            msgs::QueryMsg::DataRequest(msg) => msg.query(deps, env),
            msgs::QueryMsg::Staking(msg) => msg.query(deps, env),
            msgs::QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}

#[cw_serde]
#[serde(untagged)]
pub enum SudoMsg {
    DataRequest(data_requests::sudo::SudoMsg),
}

impl SudoMsg {
    pub fn execute(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::DataRequest(sudo) => sudo.execute(deps, env),
        }
    }
}
