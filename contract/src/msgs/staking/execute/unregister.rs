use super::{state::STAKERS, *};
use crate::{
    contract::CONTRACT_VERSION,
    crypto::{hash, verify_proof},
    error::ContractError,
    types::PublicKey,
};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
}

impl Execute {
    /// Unregisters a staker, with the requirement that no tokens are staked or pending withdrawal.
    pub fn execute(self, deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = hash(["unregister".as_bytes()]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // require that the executor has no staked or tokens pending withdrawal
        let executor = STAKERS.load(deps.storage, &self.public_key)?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        STAKERS.remove(deps.storage, &self.public_key);

        Ok(Response::new().add_attribute("action", "unregister").add_event(
            Event::new("seda-unregister").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", hex::encode(self.public_key)),
            ]),
        ))
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Unregister(value).into()
    }
}
