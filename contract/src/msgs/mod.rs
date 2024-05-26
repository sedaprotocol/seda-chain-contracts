use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;

pub mod data_requests;
pub use data_requests::ExecuteMsg as DrExecuteMsg;
pub mod staking;
pub use staking::{ExecuteMsg as StakingExecuteMsg, QueryMsg as StakingQueryMsg};
pub mod owner;
pub use owner::{ExecuteMsg as OwnerExecuteMsg, QueryMsg as OwnerQueryMsg};

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(DrExecuteMsg),
    Staking(StakingExecuteMsg),
    Owner(OwnerExecuteMsg),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::DataRequest(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Staking(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Owner(_) => todo!(),
        }
    }
}

impl From<StakingExecuteMsg> for ExecuteMsg {
    fn from(value: StakingExecuteMsg) -> Self {
        Self::Staking(value)
    }
}

impl From<OwnerExecuteMsg> for ExecuteMsg {
    fn from(value: OwnerExecuteMsg) -> Self {
        Self::Owner(value)
    }
}

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cw_serde]
#[serde(untagged)]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Staking(StakingQueryMsg),
    Owner(OwnerQueryMsg),
}

impl QueryMsg {
    pub fn query(self, deps: Deps, env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::Staking(msg) => msg.query(deps, env),
            QueryMsg::Owner(msg) => msg.query(deps, env),
        }
    }
}

impl From<StakingQueryMsg> for QueryMsg {
    fn from(value: StakingQueryMsg) -> Self {
        Self::Staking(value)
    }
}

impl From<OwnerQueryMsg> for QueryMsg {
    fn from(value: OwnerQueryMsg) -> Self {
        Self::Owner(value)
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub owner: String,
}
