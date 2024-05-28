use super::*;

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(DataRequest)]
    GetDataRequest { dr_id: Hash },
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetDataRequest { dr_id } => to_json_binary(&state::may_load_req(deps.storage, &dr_id)?),
        }
    }
}

#[cfg(test)]
impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::DataRequest(value)
    }
}
