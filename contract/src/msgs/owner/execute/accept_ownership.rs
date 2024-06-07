use super::*;

impl ExecuteHandler for execute::accept_ownership::Execute {
    /// Accept transfer contract ownership (previously triggered by owner)
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let pending_owner = PENDING_OWNER.load(deps.storage)?;
        if pending_owner.is_none() {
            return Err(ContractError::NoPendingOwnerFound);
        }
        if pending_owner.is_some_and(|owner| owner != info.sender) {
            return Err(ContractError::NotPendingOwner);
        }
        OWNER.save(deps.storage, &info.sender)?;
        PENDING_OWNER.save(deps.storage, &None)?;

        Ok(Response::new()
            .add_attribute("action", "accept-ownership")
            .add_events([Event::new("seda-accept-ownership")
                .add_attributes([("version", CONTRACT_VERSION), ("new_owner", info.sender.as_ref())])]))
    }
}
