use super::*;

#[cw_serde]
pub struct Execute {
    /// The public key of the person.
    pub(in crate::msgs::owner) public_key: PublicKey,
}

impl Execute {
    /// Remove a `Secp256k1PublicKey` to the allow list
    pub fn execute(self, deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        // remove the address from the allowlist
        ALLOWLIST.remove(deps.storage, &self.public_key);

        Ok(Response::new()
            .add_attribute("action", "remove-from-allowlist")
            .add_event(Event::new("remove-from-allowlist").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("pub_key", hex::encode(self.public_key)),
            ])))
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::RemoveFromAllowlist(value).into()
    }
}
