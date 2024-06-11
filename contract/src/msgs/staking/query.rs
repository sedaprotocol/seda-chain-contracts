pub use seda_common::msgs::staking::query::QueryMsg;
use seda_common::msgs::staking::StakerAndSeq;

use super::*;
use crate::state::get_seq;

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetStaker { public_key: executor } => to_json_binary(&utils::get_staker(deps, &executor)?),
            QueryMsg::GetAccountSeq { public_key } => {
                let seq: Uint128 = get_seq(deps.storage, &public_key)?.into();
                to_json_binary(&seq)
            }
            QueryMsg::GetStakerAndSeq { public_key } => {
                let staker = utils::get_staker(deps, &public_key)?;
                let seq: Uint128 = get_seq(deps.storage, &public_key)?.into();
                to_json_binary(&StakerAndSeq { staker, seq })
            }
            QueryMsg::IsExecutorEligible { public_key: executor } => {
                to_json_binary(&utils::is_executor_eligible(deps, executor)?)
            }
            QueryMsg::GetStakingConfig {} => to_json_binary(&state::CONFIG.load(deps.storage)?),
        }
    }
}
