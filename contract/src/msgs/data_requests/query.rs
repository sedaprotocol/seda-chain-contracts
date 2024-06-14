use super::{msgs::data_requests::query::QueryMsg, *};

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetDataRequest { dr_id } => {
                to_json_binary(&state::may_get_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestCommitment { dr_id, public_key } => {
                let dr = state::load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.get_commitment(&public_key))?
            }
            QueryMsg::GetDataRequestCommitments { dr_id } => {
                let dr = state::load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.commits)?
            }
            QueryMsg::GetDataRequestReveal { dr_id, public_key } => {
                let dr = state::load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.get_reveal(&public_key))?
            }
            QueryMsg::GetDataRequestReveals { dr_id } => {
                let dr = state::load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.reveals)?
            }
            QueryMsg::GetDataResult { dr_id } => {
                to_json_binary(&state::load_result(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestsByStatus { status, offset, limit } => {
                to_json_binary(&state::requests_by_status(deps.storage, status, offset, limit)?)?
            }
        };

        Ok(binary)
    }
}
