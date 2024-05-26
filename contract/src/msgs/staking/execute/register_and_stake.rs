use super::{
    state::{CONFIG, STAKERS},
    utils::is_staker_allowed,
    *,
};
use crate::{
    contract::CONTRACT_VERSION,
    crypto::{hash, verify_proof},
    error::ContractError,
    msgs::staking::Staker,
    state::TOKEN,
    types::{Hasher, PublicKey},
    utils::get_attached_funds,
};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) public_key: PublicKey,
    pub(in crate::msgs::staking) proof:      Vec<u8>,
    pub(in crate::msgs::staking) memo:       Option<String>,
}

impl Execute {
    /// Registers a staker with an optional p2p multi address, requiring a token deposit.
    pub fn execute(self, deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = if let Some(m) = self.memo.as_ref() {
            hash(["register_and_stake".as_bytes(), &m.hash()])
        } else {
            hash(["register_and_stake".as_bytes()])
        };

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &self.public_key)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        let minimum_stake_to_register = CONFIG.load(deps.storage)?.minimum_stake_to_register;
        if amount < minimum_stake_to_register {
            return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
        }

        let executor = Staker {
            memo:                      self.memo.clone(),
            tokens_staked:             amount,
            tokens_pending_withdrawal: 0,
        };
        STAKERS.save(deps.storage, &self.public_key, &executor)?;

        Ok(Response::new().add_attribute("action", "register-and-stake").add_event(
            Event::new("seda-register-and-stake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", hex::encode(self.public_key)),
                ("sender", info.sender.to_string()),
                ("memo", self.memo.unwrap_or_default()),
                ("tokens_staked", amount.to_string()),
                ("tokens_pending_withdrawal", "0".to_string()),
            ]),
        ))
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::RegisterAndStake(value).into()
    }
}
