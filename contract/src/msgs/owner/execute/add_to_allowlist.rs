use super::*;

impl ExecuteHandler for execute::add_to_allowlist::Execute {
    /// Add a `Secp256k1PublicKey` to the allow list
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // add the address to the allowlist
        ALLOWLIST.save(deps.storage, &self.public_key, &true)?;

        Ok(Response::new().add_attribute("action", "add-to-allowlist").add_event(
            Event::new("add-to-allowlist").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("pub_key", hex::encode(self.public_key)),
            ]),
        ))
    }
}
