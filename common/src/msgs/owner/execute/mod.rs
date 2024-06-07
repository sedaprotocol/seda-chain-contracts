use super::*;

pub mod accept_ownership;
pub mod add_to_allowlist;
pub mod remove_from_allowlist;
pub mod transfer_ownership;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum ExecuteMsg {
    TransferOwnership(transfer_ownership::Execute),
    AcceptOwnership(accept_ownership::Execute),
    /// Add a user to the allowlist.
    AddToAllowlist(add_to_allowlist::Execute),
    /// Remove a user from the allowlist.
    RemoveFromAllowlist(remove_from_allowlist::Execute),
}

impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Owner(value)
    }
}
