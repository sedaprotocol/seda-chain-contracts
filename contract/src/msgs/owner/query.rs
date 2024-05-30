use super::{
    state::{OWNER, PENDING_OWNER},
    *,
};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    GetOwner {},
    #[returns(Option<cosmwasm_std::Addr>)]
    GetPendingOwner {},
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetOwner {} => to_json_binary(&OWNER.load(deps.storage)?),
            QueryMsg::GetPendingOwner {} => to_json_binary(&PENDING_OWNER.load(deps.storage)?),
        }
    }
}

#[cfg(test)]
impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::Owner(value)
    }
}
