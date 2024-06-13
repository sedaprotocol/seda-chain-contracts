use super::*;
use crate::state::*;

impl ExecuteHandler for execute::unstake::Execute {
    /// Unstakes tokens from a given staker, to be withdrawn after a delay.
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        // compute message hash
        let message_hash = hash([
            "unstake".as_bytes(),
            &self.amount.to_be_bytes(),
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
            &inc_get_seq(deps.storage, &public_key)?.to_be_bytes(),
        ]);

        // verify the proof
        let proof = Vec::<u8>::from_hex_str(&self.proof)?;
        verify_proof(&public_key, &proof, message_hash)?;

        // error if amount is greater than staked tokens
        let mut executor = state::STAKERS.load(deps.storage, &public_key)?;
        if self.amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(executor.tokens_staked, self.amount));
        }

        // update the executor
        executor.tokens_staked -= self.amount;
        executor.tokens_pending_withdrawal += self.amount;
        state::STAKERS.save(deps.storage, &public_key, &executor)?;

        // TODO: emit when pending tokens can be withdrawn

        let executor_hex = hex::encode(self.public_key);

        Ok(Response::new().add_attribute("action", "unstake").add_events([
            Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", executor_hex.clone()),
                ("tokens_staked", executor.tokens_staked.to_string()),
                (
                    "tokens_pending_withdrawal",
                    executor.tokens_pending_withdrawal.to_string(),
                ),
                ("memo", executor.memo.map(|m| m.to_base64()).unwrap_or_default()),
            ]),
            Event::new("seda-data-request-executor-unstake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", executor_hex),
                ("amount_unstaked", self.amount.to_string()),
            ]),
        ]))
    }
}
