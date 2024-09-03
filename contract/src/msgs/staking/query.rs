pub use seda_common::msgs::staking::query::QueryMsg;
use seda_common::msgs::staking::StakerAndSeq;
use state::STAKERS;

use super::*;
use crate::state::get_seq;

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetStaker { public_key: executor } => {
                to_json_binary(&STAKERS.may_get_staker(deps.storage, &PublicKey::from_hex_str(&executor)?)?)?
            }
            QueryMsg::GetAccountSeq { public_key } => {
                let seq: Uint128 = get_seq(deps.storage, &PublicKey::from_hex_str(&public_key)?)?.into();
                to_json_binary(&seq)?
            }
            QueryMsg::GetStakerAndSeq { public_key } => {
                let public_key = PublicKey::from_hex_str(&public_key)?;
                let staker = STAKERS.may_get_staker(deps.storage, &public_key)?;
                let seq: Uint128 = get_seq(deps.storage, &public_key)?.into();
                to_json_binary(&StakerAndSeq { staker, seq })?
            }
            QueryMsg::IsStakerExecutor { public_key } => {
                to_json_binary(&STAKERS.is_staker_executor(deps.storage, &PublicKey::from_hex_str(&public_key)?)?)?
            }
            QueryMsg::IsExecutorEligible(query) => {
                to_json_binary(&STAKERS.is_executor_eligible(deps.storage, env, query)?)?
            }
            QueryMsg::GetStakingConfig {} => to_json_binary(&state::CONFIG.load(deps.storage)?)?,
        };

        Ok(binary)
    }
}
