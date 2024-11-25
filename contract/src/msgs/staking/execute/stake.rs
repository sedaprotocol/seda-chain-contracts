use owner::utils::is_staker_allowed;
use staking_events::{create_executor_action_event, create_executor_event};

use super::*;
use crate::{state::*, utils::get_attached_funds};

impl ExecuteHandler for execute::stake::Execute {
    /// Stakes with an optional memo field, requiring a token deposit.
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // verify the proof
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        let seq = inc_get_seq(deps.storage, &public_key)?;
        self.verify(public_key.as_ref(), &chain_id, env.contract.address.as_str(), seq)?;

        // if allowlist is on, check if the signer is in the allowlist
        is_staker_allowed(&deps, &public_key)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        // fetch executor from state
        let executor = match state::STAKERS.may_get_staker(deps.storage, &public_key)? {
            // new executor
            None => {
                let minimum_stake_to_register = state::STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                if amount < minimum_stake_to_register {
                    return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
                }

                let executor = Staker {
                    memo:                      self.memo.clone(),
                    tokens_staked:             amount,
                    tokens_pending_withdrawal: Uint128::zero(),
                };
                state::STAKERS.insert(deps.storage, public_key, &executor)?;
                executor
            }
            // already existing executor
            Some(mut executor) => {
                let minimum_stake_to_register = state::STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                if amount + executor.tokens_staked < minimum_stake_to_register {
                    return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
                }
                executor.tokens_staked += amount;

                state::STAKERS.update(deps.storage, public_key, &executor)?;
                executor
            }
        };

        Ok(Response::new().add_attribute("action", "stake").add_events([
            create_executor_action_event("stake", self.public_key.clone(), info.sender.to_string(), amount, seq),
            create_executor_event(executor, self.public_key),
        ]))
    }
}
