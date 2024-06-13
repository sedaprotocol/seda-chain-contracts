use super::*;

impl ExecuteHandler for execute::remove_from_allowlist::Execute {
    /// Remove a `Secp256k1PublicKey` to the allow list
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // remove the address from the allowlist
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        ALLOWLIST.remove(deps.storage, &public_key);

        Ok(Response::new()
            .add_attribute("action", "remove-from-allowlist")
            .add_event(
                Event::new("remove-from-allowlist")
                    .add_attributes([("version", CONTRACT_VERSION.to_string()), ("pub_key", self.public_key)]),
            ))
    }
}
