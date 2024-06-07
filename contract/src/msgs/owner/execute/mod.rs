use state::{ALLOWLIST, OWNER, PENDING_OWNER};

use super::{
    msgs::owner::execute::{self, ExecuteMsg},
    *,
};

pub(in crate::msgs::owner) mod accept_ownership;
pub(in crate::msgs::owner) mod add_to_allowlist;
pub(in crate::msgs::owner) mod remove_from_allowlist;
pub(in crate::msgs::owner) mod transfer_ownership;

impl ExecuteHandler for ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::TransferOwnership(msg) => msg.execute(deps, env, info),
            ExecuteMsg::AcceptOwnership(msg) => msg.execute(deps, env, info),
            ExecuteMsg::AddToAllowlist(msg) => msg.execute(deps, env, info),
            ExecuteMsg::RemoveFromAllowlist(msg) => msg.execute(deps, env, info),
        }
    }
}
