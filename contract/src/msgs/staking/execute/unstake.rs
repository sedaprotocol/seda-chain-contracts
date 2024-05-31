use super::{state::STAKERS, *};
use crate::crypto::{hash, verify_proof};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
    pub(in crate::msgs::staking) amount:     Uint128,
}

impl Execute {
    /// Unstakes tokens from a given staker, to be withdrawn after a delay.
    pub fn execute(self, deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = hash(["unstake".as_bytes(), &self.amount.to_be_bytes()]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // error if amount is greater than staked tokens
        let mut executor = STAKERS.load(deps.storage, &self.public_key)?;
        if self.amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(executor.tokens_staked, self.amount));
        }

        // update the executor
        executor.tokens_staked -= self.amount;
        executor.tokens_pending_withdrawal += self.amount;
        STAKERS.save(deps.storage, &self.public_key, &executor)?;

        // TODO: emit when pending tokens can be withdrawn

        let executor_hex = hex::encode(self.public_key);
        let mut event = Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("executor", executor_hex.clone()),
            ("tokens_staked", executor.tokens_staked.to_string()),
            (
                "tokens_pending_withdrawal",
                executor.tokens_pending_withdrawal.to_string(),
            ),
        ]);
        // https://github.com/CosmWasm/cosmwasm/issues/2163
        if let Some(memo) = executor.memo {
            event = event.add_attribute("memo", memo);
        }

        Ok(Response::new().add_attribute("action", "unstake").add_events([
            event,
            Event::new("seda-data-request-executor-unstake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", executor_hex),
                ("amount_unstaked", self.amount.to_string()),
            ]),
        ]))
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Unstake(value).into()
    }
}
