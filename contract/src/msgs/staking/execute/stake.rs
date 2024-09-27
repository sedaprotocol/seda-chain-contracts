use owner::utils::is_staker_allowed;

use super::*;
use crate::{state::*, utils::get_attached_funds};

impl ExecuteHandler for execute::stake::Execute {
    /// Stakes with an optional memo field, requiring a token deposit.
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

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &public_key)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        // fetch executor from state
        match state::STAKERS.may_get_staker(deps.storage, &public_key)? {
            // new executor
            None => {
                let minimum_stake_to_register = state::STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                if amount < minimum_stake_to_register {
                    return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
                }

                state::STAKERS.insert(
                    deps.storage,
                    public_key,
                    &Staker {
                        memo:                      self.memo.clone(),
                        tokens_staked:             amount,
                        tokens_pending_withdrawal: Uint128::zero(),
                    },
                )
            }
            // already existing executor
            Some(mut executor) => {
                let minimum_stake_to_register = state::STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                if amount + executor.tokens_staked < minimum_stake_to_register {
                    return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
                }
                executor.tokens_staked += amount;

                state::STAKERS.update(deps.storage, public_key, &executor)
            }
        }?;

        Ok(Response::new().add_attribute("action", "stake").add_event(
            Event::new("seda-register-and-stake").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("executor", self.public_key.clone()),
                ("sender", info.sender.to_string()),
                ("tokens_staked", amount.to_string()),
                ("tokens_pending_withdrawal", "0".to_string()),
                ("memo", self.memo.map(|b| b.to_base64()).unwrap_or_default()),
            ]),
        ))
    }
}
