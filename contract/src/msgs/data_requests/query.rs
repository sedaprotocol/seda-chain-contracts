use super::{msgs::data_requests::query::QueryMsg, *};

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, _env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetDataRequest { dr_id } => {
                to_json_binary(&state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestCommitment { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.as_ref().map(|dr| dr.get_commitment(&public_key)))?
            }
            QueryMsg::GetDataRequestCommitments { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let commitments = dr.map(|dr| dr.commits).unwrap_or_default();
                to_json_binary(&commitments)?
            }
            QueryMsg::GetDataRequestReveal { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.as_ref().map(|dr| dr.get_reveal(&public_key)))?
            }
            QueryMsg::GetDataRequestReveals { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let reveals = dr.map(|dr| dr.reveals).unwrap_or_default();
                to_json_binary(&reveals)?
            }
            QueryMsg::GetDataResult { dr_id } => {
                to_json_binary(&state::may_load_result(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestsByStatus { status, offset, limit } => {
                to_json_binary(&state::requests_by_status(deps.storage, &status, offset, limit)?)?
            }
        };

        Ok(binary)
    }
}
