use staking_events::{create_executor_action_event, create_executor_event};

use super::*;
use crate::state::*;

impl ExecuteHandler for execute::unstake::Execute {
    /// Unstakes tokens from a given staker, to be withdrawn after a delay.
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // verify the proof
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        let seq = inc_get_seq(deps.storage, &public_key)?;
        self.verify(public_key.as_ref(), &chain_id, env.contract.address.as_str(), seq)?;

        // error if amount is greater than staked tokens
        let mut executor = state::STAKERS.get_staker(deps.storage, &public_key)?;
        if self.amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(executor.tokens_staked, self.amount));
        }

        // update the executor
        executor.tokens_staked -= self.amount;
        executor.tokens_pending_withdrawal += self.amount;
        state::STAKERS.update(deps.storage, public_key, &executor)?;

        // TODO: emit when pending tokens can be withdrawn

        Ok(Response::new().add_attribute("action", "unstake").add_events([
            create_executor_action_event(
                "seda-executor-action-unstake",
                self.public_key.clone(),
                info.sender.to_string(),
                self.amount,
                seq,
            ),
            create_executor_event(executor, self.public_key),
        ]))
    }
}
