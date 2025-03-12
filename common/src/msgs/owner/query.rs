#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(cosmwasm_schema::QueryResponses))]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum QueryMsg {
    #[cfg_attr(feature = "cosmwasm", returns(cosmwasm_std::Addr))]
    GetOwner {},
    #[cfg_attr(feature = "cosmwasm", returns(Option<cosmwasm_std::Addr>))]
    GetPendingOwner {},
    #[cfg_attr(feature = "cosmwasm", returns(bool))]
    IsPaused {},
}

impl From<QueryMsg> for crate::msgs::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Owner(value)
    }
}
