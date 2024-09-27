use super::{state::ALLOWLIST, *};
use crate::msgs::staking::state::STAKING_CONFIG;

pub fn is_staker_allowed(deps: &DepsMut, public_key: &PublicKey) -> Result<(), ContractError> {
    let allowlist_enabled = STAKING_CONFIG.load(deps.storage)?.allowlist_enabled;
    if allowlist_enabled {
        let is_allowed = ALLOWLIST.may_load(deps.storage, public_key)?;
        if is_allowed.is_none() {
            return Err(ContractError::NotOnAllowlist);
        }
    }

    Ok(())
}
