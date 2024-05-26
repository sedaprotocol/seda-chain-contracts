use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::PublicKey;

pub mod data_requests;
pub use data_requests::ExecuteMsg as DrExecuteMsg;
pub mod staking;
pub use staking::{ExecuteMsg as StakingExecuteMsg, QueryMsg as StakingQueryMsg};

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    DataRequest(DrExecuteMsg),
    Staking(StakingExecuteMsg),
    Owner(OwnerExecuteMsg),
}

#[cw_serde]
#[serde(untagged)]
pub enum OwnerExecuteMsg {
    TransferOwnership {
        new_owner: String,
    },
    AcceptOwnership {},
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

impl From<StakingExecuteMsg> for ExecuteMsg {
    fn from(value: StakingExecuteMsg) -> Self {
        Self::Staking(value)
    }
}

impl From<OwnerExecuteMsg> for ExecuteMsg {
    fn from(value: OwnerExecuteMsg) -> Self {
        Self::Owner(value)
    }
}

// https://github.com/CosmWasm/cosmwasm/issues/2030
#[cw_serde]
#[serde(untagged)]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Staking(StakingQueryMsg),
    Owner(OwnerQueryMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum OwnerQueryMsg {
    #[returns(cosmwasm_std::Addr)]
    GetOwner,
    #[returns(Option<cosmwasm_std::Addr>)]
    GetPendingOwner,
}

impl From<StakingQueryMsg> for QueryMsg {
    fn from(value: StakingQueryMsg) -> Self {
        Self::Staking(value)
    }
}

impl From<OwnerQueryMsg> for QueryMsg {
    fn from(value: OwnerQueryMsg) -> Self {
        Self::Owner(value)
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
    pub owner: String,
}
