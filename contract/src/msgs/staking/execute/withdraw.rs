use cosmwasm_std::{coins, BankMsg};

use super::{state::STAKERS, *};
use crate::{
    crypto::{hash, verify_proof},
    state::TOKEN,
};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
    pub(in crate::msgs::staking) amount:     u128,
}

impl Execute {
    /// Sends tokens back to the sender that are marked as pending withdrawal.
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = hash(["withdraw".as_bytes(), &self.amount.to_be_bytes()]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor = STAKERS.load(deps.storage, &self.public_key)?;
        if self.amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                self.amount,
            ));
        }

        // update the executor
        executor.tokens_pending_withdrawal -= self.amount;
        STAKERS.save(deps.storage, &self.public_key, &executor)?;

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount:     coins(self.amount, token),
        };

        let sender = info.sender.into_string();
        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
                    ("version", CONTRACT_VERSION.to_string()),
                    ("executor", sender.clone()),
                    ("memo", executor.memo.unwrap_or_default()),
                    ("tokens_staked", executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-withdraw").add_attributes([
                    ("version", CONTRACT_VERSION.to_string()),
                    ("executor", sender),
                    ("amount_withdrawn", self.amount.to_string()),
                ]),
            ]))
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Withdraw(value).into()
    }
}
