use super::{state::STAKERS, utils::is_staker_allowed, *};
use crate::{
    contract::CONTRACT_VERSION,
    crypto::{hash, verify_proof},
    error::ContractError,
    state::TOKEN,
    types::PublicKey,
    utils::get_attached_funds,
};

#[cw_serde]
pub struct Execute {
    pub public_key: PublicKey,
    pub proof:      Vec<u8>,
}

impl Execute {
    /// Deposits and stakes tokens for an already existing staker.
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        // compute message hash
        let message_hash = hash(["increase_stake".as_bytes()]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &self.public_key)?;

        // update staked tokens for executor
        let mut executor = STAKERS.load(deps.storage, &self.public_key)?;
        executor.tokens_staked += amount;
        STAKERS.save(deps.storage, &self.public_key, &executor)?;

        Ok(Response::new().add_attribute("action", "increase-stake").add_events([
            Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", hex::encode(&self.public_key)),
                ("memo", executor.memo.unwrap_or_default()),
                ("tokens_staked", executor.tokens_staked.to_string()),
                (
                    "tokens_pending_withdrawal",
                    executor.tokens_pending_withdrawal.to_string(),
                ),
            ]),
            Event::new("seda-data-request-executor-increase-stake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", hex::encode(self.public_key)),
                ("amount_deposited", amount.to_string()),
            ]),
        ]))
    }
}

impl From<Execute> for ExecuteMsg {
    fn from(value: Execute) -> Self {
        ExecuteMsg::IncreaseStake(value)
    }
}
