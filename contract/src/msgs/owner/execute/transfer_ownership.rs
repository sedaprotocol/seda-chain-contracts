use super::*;

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::owner) new_owner: String,
}

impl Execute {
    /// Start 2-step process for transfer contract ownership to a new address
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        PENDING_OWNER.save(deps.storage, &Some(deps.api.addr_validate(&self.new_owner)?))?;

        Ok(Response::new()
            .add_attribute("action", "transfer_ownership")
            .add_events([Event::new("seda-transfer-ownership").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("sender", info.sender.into_string()),
                ("pending_owner", self.new_owner),
            ])]))
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::TransferOwnership(value).into()
    }
}
