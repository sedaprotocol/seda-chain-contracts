pub mod allow_list {
    use common::error::ContractError;
    #[cfg(not(feature = "library"))]
    use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response};

    use crate::{
        state::{ALLOWLIST, OWNER},
        utils::validate_sender,
    };

    pub fn add_to_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        sender: Option<String>,
        address: Addr,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if sender != owner {
            return Err(ContractError::NotOwner);
        }

        // add the address to the allowlist
        ALLOWLIST.save(deps.storage, address, &true)?;

        Ok(Response::new())
    }

    pub fn remove_from_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        sender: Option<String>,
        address: Addr,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if sender != owner {
            return Err(ContractError::NotOwner);
        }

        // remove the address from the allowlist
        ALLOWLIST.remove(deps.storage, address);

        Ok(Response::new())
    }
}
