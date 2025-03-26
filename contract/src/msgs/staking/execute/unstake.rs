use staking_events::{create_executor_action_event, create_executor_event};

use super::*;
use crate::state::*;

impl ExecuteHandler for execute::unstake::Execute {
    /// Unstakes all tokens from a given staker.
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // verify the proof
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        let seq = inc_get_seq(deps.storage, &public_key)?;
        self.verify(public_key.as_ref(), &chain_id, env.contract.address.as_str(), seq)?;

        let mut executor = state::STAKERS.get_staker(deps.storage, &public_key)?;

        // update the executor
        let amount = executor.tokens_staked;
        executor.tokens_staked -= amount;
        executor.tokens_pending_withdrawal += amount;
        state::STAKERS.update(deps.storage, public_key, &executor)?;

        // TODO: emit when pending tokens can be withdrawn

        Ok(Response::new().add_attribute("action", "unstake").add_events([
            create_executor_action_event("unstake", self.public_key.clone(), info.sender.to_string(), amount, seq),
            create_executor_event(executor, self.public_key),
        ]))
    }
}
