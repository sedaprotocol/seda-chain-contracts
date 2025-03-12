pub mod accept_ownership;
pub mod add_to_allowlist;
pub mod pause;
pub mod remove_from_allowlist;
pub mod transfer_ownership;
pub mod unpause;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum ExecuteMsg {
    TransferOwnership(transfer_ownership::Execute),
    AcceptOwnership(accept_ownership::Execute),
    /// Add a user to the allowlist.
    AddToAllowlist(add_to_allowlist::Execute),
    /// Remove a user from the allowlist.
    RemoveFromAllowlist(remove_from_allowlist::Execute),
    Pause(pause::Execute),
    Unpause(unpause::Execute),
}

impl From<ExecuteMsg> for crate::msgs::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Owner(value)
    }
}
