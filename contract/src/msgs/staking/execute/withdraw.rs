use cosmwasm_std::{coins, BankMsg};

use super::*;
use crate::{
    crypto::{hash, verify_proof},
    state::{CHAIN_ID, TOKEN},
};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
    pub(in crate::msgs::staking) amount:     Uint128,
}

impl Execute {
    /// Sends tokens back to the sender that are marked as pending withdrawal.
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let chain_id = CHAIN_ID.load(deps.storage)?;
        // compute message hash
        let message_hash = hash([
            "withdraw".as_bytes(),
            &self.amount.to_be_bytes(),
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
            &state::inc_get_seq(deps.storage, &self.public_key)?.to_be_bytes(),
        ]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor = state::STAKERS.load(deps.storage, &self.public_key)?;
        if self.amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                self.amount,
            ));
        }

        // update the executor
        executor.tokens_pending_withdrawal -= self.amount;
        state::STAKERS.save(deps.storage, &self.public_key, &executor)?;

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount:     coins(self.amount.u128(), token),
        };

        let sender = info.sender.into_string();
        let mut event = Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("executor", sender.clone()),
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

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Withdraw(value).into()
    }
}
