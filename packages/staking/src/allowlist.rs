pub mod allow_list {
    use common::{error::ContractError, types::Secpk256k1PublicKey};
    #[cfg(not(feature = "library"))]
    use cosmwasm_std::{DepsMut, MessageInfo, Response};

    use crate::state::{ALLOWLIST, OWNER};

    pub fn add_to_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        pub_key: Secpk256k1PublicKey,
    ) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // add the address to the allowlist
        ALLOWLIST.save(deps.storage, &pub_key, &true)?;

        Ok(Response::new())
    }

    pub fn remove_from_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        pub_key: Secpk256k1PublicKey,
    ) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // remove the address from the allowlist
        ALLOWLIST.remove(deps.storage, &pub_key);

        Ok(Response::new())
    }
}
