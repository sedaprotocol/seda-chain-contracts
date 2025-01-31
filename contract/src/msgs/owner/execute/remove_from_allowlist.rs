use super::*;
use crate::msgs::staking::state::STAKERS;

impl ExecuteHandler for execute::remove_from_allowlist::Execute {
    /// Remove a `Secp256k1PublicKey` to the allow list
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // we need to remove the address from the allowlist
        let public_key = PublicKey::from_hex_str(&self.public_key)?;

        if let Some(staker) = STAKERS.may_get_staker(deps.storage, &public_key)? {
            // we move their staked tokens to the pending withdrawal
            // so that they can withdraw them and no longer be a staker
            let staker = Staker {
                memo:                      staker.memo,
                tokens_staked:             Uint128::new(0),
                tokens_pending_withdrawal: staker.tokens_staked.checked_add(staker.tokens_pending_withdrawal)?,
            };

            STAKERS.update(deps.storage, public_key.clone(), &staker)?;
        }

        // do this at the end in case we fail above
        ALLOWLIST.remove(deps.storage, &public_key);

        Ok(Response::new()
            .add_attribute("action", "remove-from-allowlist")
            .add_event(Event::new("seda-contract").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("identity", self.public_key),
                ("action", "allowlist-remove".to_string()),
            ])))
    }
}
