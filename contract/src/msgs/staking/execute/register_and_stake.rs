use seda_contract_common::msgs::staking::Staker;

use self::staking::owner::utils::is_staker_allowed;
use super::*;
use crate::{
    crypto::{hash, verify_proof},
    state::{inc_get_seq, CHAIN_ID, TOKEN},
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
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let chain_id = CHAIN_ID.load(deps.storage)?;
        // compute message hash
        let message_hash = hash([
            "register_and_stake".as_bytes(),
            &self.memo.hash(),
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
            &inc_get_seq(deps.storage, &self.public_key)?.to_be_bytes(),
        ]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &self.public_key)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        let minimum_stake_to_register = state::CONFIG.load(deps.storage)?.minimum_stake_to_register;
        if amount < minimum_stake_to_register {
            return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
        }

        let executor = Staker {
            memo:                      self.memo.clone(),
            tokens_staked:             amount,
            tokens_pending_withdrawal: Uint128::zero(),
        };
        state::STAKERS.save(deps.storage, &self.public_key, &executor)?;

        let mut event = Event::new("seda-register-and-stake").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("executor", hex::encode(self.public_key)),
            ("sender", info.sender.to_string()),
            ("tokens_staked", amount.to_string()),
            ("tokens_pending_withdrawal", "0".to_string()),
        ]);
        // https://github.com/CosmWasm/cosmwasm/issues/2163
        if let Some(memo) = self.memo {
            event = event.add_attribute("memo", memo);
        }

        Ok(Response::new()
            .add_attribute("action", "register-and-stake")
            .add_event(event))
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::RegisterAndStake(value).into()
    }
}
