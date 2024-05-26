use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use super::{state::CONFIG, utils};
use crate::types::PublicKey;

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(PublicKey)]
    GetStaker { executor: PublicKey },
    #[returns(bool)]
    IsExecutorEligible { executor: PublicKey },
    #[returns(super::StakingConfig)]
    GetStakingConfig,
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetStaker { executor } => to_json_binary(&utils::get_staker(deps, executor)?),
            QueryMsg::IsExecutorEligible { executor } => to_json_binary(&utils::is_executor_eligible(deps, executor)?),
            QueryMsg::GetStakingConfig => to_json_binary(&CONFIG.load(deps.storage)?),
        }
    }
}

impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Staking(value)
    }
}
