use self::state::ALLOWLIST;
use super::*;
use crate::{error::ContractError, msgs::staking::state::CONFIG, types::PublicKey};

pub fn is_staker_allowed(deps: &DepsMut, public_key: &PublicKey) -> Result<(), ContractError> {
    let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
    if allowlist_enabled {
        let is_allowed = ALLOWLIST.may_load(deps.storage, public_key)?;
        if is_allowed.is_none() {
            return Err(ContractError::NotOnAllowlist);
        }
    }

    Ok(())
}
