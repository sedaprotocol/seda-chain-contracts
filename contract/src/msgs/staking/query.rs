use super::{state::CONFIG, *};

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

#[cfg(test)]
impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Staking(value)
    }
}
