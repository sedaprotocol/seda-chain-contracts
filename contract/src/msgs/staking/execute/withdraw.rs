use super::*;
use crate::state::*;

impl ExecuteHandler for execute::withdraw::Execute {
    /// Sends tokens back to the sender that are marked as pending withdrawal.
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // verify the proof
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        self.verify(
            public_key.as_ref(),
            &chain_id,
            env.contract.address.as_str(),
            inc_get_seq(deps.storage, &public_key)?,
        )?;

        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor = state::STAKERS.get_staker(deps.storage, &public_key)?;
        if self.amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                self.amount,
            ));
        }

        // update the executor (remove if balances are zero)
        executor.tokens_pending_withdrawal -= self.amount;
        if executor.tokens_pending_withdrawal.is_zero() && executor.tokens_staked.is_zero() {
            state::STAKERS.remove(deps.storage, public_key)?;
        } else {
            state::STAKERS.update(deps.storage, public_key, &executor)?;
        }

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount:     coins(self.amount.u128(), token),
        };

        let sender = info.sender.into_string();
        let event = Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("executor", sender.clone()),
            ("tokens_staked", executor.tokens_staked.to_string()),
            (
                "tokens_pending_withdrawal",
                executor.tokens_pending_withdrawal.to_string(),
            ),
            ("memo", executor.memo.map(|m| m.to_base64()).unwrap_or_default()),
        ]);

        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_events([
                event,
                Event::new("seda-data-request-executor-withdraw").add_attributes([
                    ("version", CONTRACT_VERSION.to_string()),
                    ("executor", sender),
                    ("amount_withdrawn", self.amount.to_string()),
                ]),
            ]))
    }
}
