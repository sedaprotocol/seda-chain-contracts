use super::{
    msgs::owner::query::QueryMsg,
    state::{OWNER, PENDING_OWNER},
    *,
};
use crate::state::PAUSED;

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetOwner {} => to_json_binary(&OWNER.load(deps.storage)?)?,
            QueryMsg::GetPendingOwner {} => to_json_binary(&PENDING_OWNER.load(deps.storage)?)?,
            QueryMsg::IsPaused {} => to_json_binary(&PAUSED.load(deps.storage)?)?,
        };

        Ok(binary)
    }
}
