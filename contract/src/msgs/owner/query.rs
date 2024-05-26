use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use super::state::{OWNER, PENDING_OWNER};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    GetOwner,
    #[returns(Option<cosmwasm_std::Addr>)]
    GetPendingOwner,
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetOwner => to_json_binary(&OWNER.load(deps.storage)?),
            QueryMsg::GetPendingOwner => to_json_binary(&PENDING_OWNER.load(deps.storage)?),
        }
    }
}
