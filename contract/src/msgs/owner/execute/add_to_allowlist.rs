use super::*;
use crate::contract::CONTRACT_VERSION;

#[cw_serde]
pub struct Execute {
    /// The public key of the person.
    pub(in crate::msgs::owner) public_key: PublicKey,
}

impl Execute {
    /// Add a `Secp256k1PublicKey` to the allow list
    pub fn execute(self, deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
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

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::AddToAllowlist(value).into()
    }
}
