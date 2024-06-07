use super::*;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(QueryResponses))]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum QueryMsg {
    #[cfg_attr(feature = "cosmwasm", returns(cosmwasm_std::Addr))]
    GetOwner {},
    #[cfg_attr(feature = "cosmwasm", returns(Option<cosmwasm_std::Addr>))]
    GetPendingOwner {},
}

impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Owner(value)
    }
}
