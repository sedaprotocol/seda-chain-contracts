use cosmwasm_std::Event;
use state::{ALLOWLIST, OWNER, PENDING_OWNER};

use super::*;

pub(in crate::msgs::owner) mod accept_ownership;
pub(in crate::msgs::owner) mod add_to_allowlist;
pub(in crate::msgs::owner) mod remove_from_allowlist;
pub(in crate::msgs::owner) mod transfer_ownership;

#[cw_serde]
pub enum ExecuteMsg {
    TransferOwnership(transfer_ownership::Execute),
    AcceptOwnership(accept_ownership::Execute),
    /// Add a user to the allowlist.
    AddToAllowlist(add_to_allowlist::Execute),
    /// Remove a user from the allowlist.
    RemoveFromAllowlist(remove_from_allowlist::Execute),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::TransferOwnership(msg) => msg.execute(deps, env, info),
            ExecuteMsg::AcceptOwnership(msg) => msg.execute(deps, env, info),
            ExecuteMsg::AddToAllowlist(msg) => msg.execute(deps, info),
            ExecuteMsg::RemoveFromAllowlist(msg) => msg.execute(deps, info),
        }
    }
}

#[cfg(test)]
impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Owner(value)
    }
}
