use super::{msgs::data_requests::query::QueryMsg, *};

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
        match self {
            QueryMsg::GetDataRequest { dr_id } => to_json_binary(&state::may_load_req(deps.storage, &dr_id)?),
            QueryMsg::GetDataRequestCommitment { dr_id, public_key } => {
                let dr = state::load_req(deps.storage, &dr_id)?;
                let public_key_str = hex::encode(public_key);
                to_json_binary(&dr.get_commitment(&public_key_str))
            }
            QueryMsg::GetDataRequestCommitments { dr_id } => {
                let dr = state::load_req(deps.storage, &dr_id)?;
                to_json_binary(&dr.commits)
            }
            QueryMsg::GetDataRequestReveal { dr_id, public_key } => {
                let dr = state::load_req(deps.storage, &dr_id)?;
                let public_key_str = hex::encode(public_key);
                to_json_binary(&dr.get_reveal(&public_key_str))
            }
            QueryMsg::GetDataRequestReveals { dr_id } => {
                let dr = state::load_req(deps.storage, &dr_id)?;
                to_json_binary(&dr.reveals)
            }
            QueryMsg::GetResolvedDataRequest { dr_id } => {
                to_json_binary(&state::load_resolved_req(deps.storage, &dr_id)?)
            }
            QueryMsg::GetDataRequestsByStatus { status, page, limit } => {
                to_json_binary(&state::requests_by_status(deps.storage, &status, page, limit)?)
            }
        }
    }
}
