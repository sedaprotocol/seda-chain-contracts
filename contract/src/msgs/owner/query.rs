use super::{
    msgs::owner::query::QueryMsg,
    state::{OWNER, PENDING_OWNER},
    *,
};

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetOwner {} => to_json_binary(&OWNER.load(deps.storage)?),
            QueryMsg::GetPendingOwner {} => to_json_binary(&PENDING_OWNER.load(deps.storage)?),
        }
    }
}
