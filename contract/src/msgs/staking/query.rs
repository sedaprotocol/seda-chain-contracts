use cw_storage_plus::Bound;
pub use seda_common::msgs::staking::query::{is_executor_eligible, QueryMsg};
use seda_common::msgs::staking::{Executor, GetExecutorsResponse, StakerAndSeq};
use state::{is_eligible_for_dr::is_eligible_for_dr, STAKERS};

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
            QueryMsg::IsExecutorEligible(query) => query.query(deps, env)?,
            QueryMsg::GetStakingConfig {} => to_json_binary(&state::STAKING_CONFIG.load(deps.storage)?)?,
            QueryMsg::GetExecutors { offset, limit } => {
                let start = Some(Bound::inclusive(offset));
                let end = Some(Bound::exclusive(offset + limit));
                let executors = STAKERS
                    .public_keys
                    .index_to_key
                    .range(deps.storage, start, end, Order::Ascending)
                    .map(|r| {
                        r.map(|(_, pub_key)| {
                            let staker = STAKERS.get_staker(deps.storage, &pub_key)?;
                            Ok(Executor {
                                public_key:                pub_key.to_hex(),
                                memo:                      staker.memo,
                                tokens_staked:             staker.tokens_staked,
                                tokens_pending_withdrawal: staker.tokens_pending_withdrawal,
                            })
                        })
                    })
                    .collect::<StdResult<StdResult<Vec<_>>>>()??;

                let response = GetExecutorsResponse { executors };
                to_json_binary(&response)?
            }
        };

        Ok(binary)
    }
}

impl QueryHandler for is_executor_eligible::Query {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        let (executor, dr_id, _) = self.parts()?;
        let executor = PublicKey(executor);

        // Validate signature
        let chain_id = crate::state::CHAIN_ID.load(deps.storage)?;
        if self
            .verify(&executor, &chain_id, env.contract.address.as_str())
            .is_err()
        {
            return Ok(to_json_binary(&false)?);
        }

        // Check DR is in data_request_pool
        if data_requests::state::load_request(deps.storage, &dr_id).is_err() {
            return Ok(to_json_binary(&false)?);
        }

        if !STAKERS.is_staker_executor(deps.storage, &executor)? {
            return Ok(to_json_binary(&false)?);
        }

        Ok(to_json_binary(&is_eligible_for_dr(deps, env, dr_id, executor)?)?)
    }
}
