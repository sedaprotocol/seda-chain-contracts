use self::staking::owner::utils::is_staker_allowed;
use super::*;
use crate::{
    crypto::{hash, verify_proof},
    state::{inc_get_seq, CHAIN_ID, TOKEN},
    utils::get_attached_funds,
};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
}

impl Execute {
    /// Deposits and stakes tokens for an already existing staker.
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;
        let chain_id = CHAIN_ID.load(deps.storage)?;

        // compute message hash
        let message_hash = hash([
            "increase_stake".as_bytes(),
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
            &inc_get_seq(deps.storage, &self.public_key)?.to_be_bytes(),
        ]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &self.public_key)?;

        // update staked tokens for executor
        let mut executor = state::STAKERS.load(deps.storage, &self.public_key)?;
        executor.tokens_staked += amount;
        state::STAKERS.save(deps.storage, &self.public_key, &executor)?;

        let mut event = Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("executor", hex::encode(&self.public_key)),
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

        Ok(Response::new().add_attribute("action", "increase-stake").add_events([
            event,
            Event::new("seda-data-request-executor-increase-stake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", hex::encode(self.public_key)),
                ("amount_deposited", amount.to_string()),
            ]),
        ]))
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::IncreaseStake(value).into()
    }
}
