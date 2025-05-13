use super::{
    msgs::owner::query::QueryMsg,
    state::{ALLOWLIST, OWNER, PENDING_OWNER},
    *,
};
use crate::state::PAUSED;

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetOwner {} => to_json_binary(&OWNER.load(deps.storage)?)?,
            QueryMsg::GetPendingOwner {} => to_json_binary(&PENDING_OWNER.load(deps.storage)?)?,
            QueryMsg::IsPaused {} => to_json_binary(&PAUSED.load(deps.storage)?)?,
            QueryMsg::GetAllowList {} => {
                let allowlist = ALLOWLIST
                    .keys_raw(deps.storage, None, None, Order::Ascending)
                    .map(|key| key.to_hex())
                    .collect::<Vec<_>>();
                to_json_binary(&allowlist)?
            }
        };

        Ok(binary)
    }
}
