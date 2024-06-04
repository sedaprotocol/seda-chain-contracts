use super::{state::CONFIG, *};

#[cw_serde]
pub struct StakerAndSeq {
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staker: Option<Staker>,
    pub seq:    Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(Option<Staker>)]
    GetStaker { executor: PublicKey },
    #[returns(Uint128)]
    GetAccountSeq { public_key: PublicKey },
    #[returns(StakerAndSeq)]
    GetStakerandSeq { public_key: PublicKey },
    #[returns(bool)]
    IsExecutorEligible { executor: PublicKey },
    #[returns(super::StakingConfig)]
    GetStakingConfig {},
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetStaker { executor } => to_json_binary(&utils::get_staker(deps, &executor)?),
            QueryMsg::GetAccountSeq { public_key } => {
                let seq: Uint128 = state::get_seq(deps.storage, &public_key)?.into();
                to_json_binary(&seq)
            }
            QueryMsg::GetStakerandSeq { public_key } => {
                let staker = utils::get_staker(deps, &public_key)?;
                let seq: Uint128 = state::get_seq(deps.storage, &public_key)?.into();
                to_json_binary(&StakerAndSeq { staker, seq })
            }
            QueryMsg::IsExecutorEligible { executor } => to_json_binary(&utils::is_executor_eligible(deps, executor)?),
            QueryMsg::GetStakingConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),
        }
    }
}

#[cfg(test)]
impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Staking(value)
    }
}
