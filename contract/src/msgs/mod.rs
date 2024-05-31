use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::*;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    contract::CONTRACT_VERSION,
    crypto::{hash, verify_proof},
    error::ContractError,
    types::*,
};

pub mod data_requests;
pub mod owner;
pub mod staking;

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(Box<data_requests::execute::ExecuteMsg>),
    Staking(staking::execute::ExecuteMsg),
    Owner(owner::execute::ExecuteMsg),
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

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cw_serde]
#[serde(untagged)]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    DataRequest(data_requests::query::QueryMsg),
    Staking(staking::query::QueryMsg),
    Owner(owner::query::QueryMsg),
}

impl QueryMsg {
    pub fn query(self, deps: Deps, env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::DataRequest(msg) => msg.query(deps, env),
            QueryMsg::Staking(msg) => msg.query(deps, env),
            QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token:    String,
    pub owner:    String,
    pub chain_id: String,
}
