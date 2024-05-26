use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{config, error::ContractError, types::PublicKey};

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    TransferOwnership {
        new_owner: String,
    },
    AcceptOwnership,
    /// Add a user to the allowlist.
    AddToAllowlist {
        /// The public key of the person to allowlist.
        pub_key: PublicKey,
    },
    /// Remove a user from the allowlist.
    RemoveFromAllowlist {
        /// The public key of the person remove from allowlist.
        pub_key: PublicKey,
    },
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::TransferOwnership { new_owner } => config::transfer_ownership(deps, env, info, new_owner),
            ExecuteMsg::AcceptOwnership => config::accept_ownership(deps, env, info),
            ExecuteMsg::AddToAllowlist { pub_key } => config::add_to_allowlist(deps, info, pub_key),
            ExecuteMsg::RemoveFromAllowlist { pub_key } => config::remove_from_allowlist(deps, info, pub_key),
        }
    }
}
