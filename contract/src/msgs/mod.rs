use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult};

use crate::error::ContractError;

pub mod data_requests;
pub mod owner;
pub mod staking;

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(data_requests::ExecuteMsg),
    Staking(staking::execute::ExecuteMsg),
    Owner(owner::ExecuteMsg),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::DataRequest(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Staking(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Owner(msg) => msg.execute(deps, env, info),
        }
    }
}

impl From<owner::ExecuteMsg> for ExecuteMsg {
    fn from(value: owner::ExecuteMsg) -> Self {
        Self::Owner(value)
    }
}

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cw_serde]
#[serde(untagged)]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Staking(staking::query::QueryMsg),
    Owner(owner::QueryMsg),
}

impl QueryMsg {
    pub fn query(self, deps: Deps, env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::Staking(msg) => msg.query(deps, env),
            QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}

impl From<owner::QueryMsg> for QueryMsg {
    fn from(value: owner::QueryMsg) -> Self {
        Self::Owner(value)
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub owner: String,
}
