use execute::commit_result::verify_commit;

use super::{
    msgs::data_requests::{execute::commit_result, query::QueryMsg},
    *,
};
use crate::state::PAUSED;

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        let contract_paused = PAUSED.load(deps.storage)?;

        let binary = match self {
            QueryMsg::CanExecutorCommit {
                dr_id,
                public_key,
                commitment,
                proof,
            } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let valid = dr.is_some_and(|dr| {
                    let commit_msg = commit_result::Execute {
                        dr_id,
                        commitment,
                        public_key,
                        proof,
                    };
                    verify_commit(deps, &env, &commit_msg, &dr).is_ok()
                });
                to_json_binary(&valid)?
            }
            QueryMsg::CanExecutorReveal { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let can_reveal = dr.map(|dr| dr.reveal_started() && dr.get_commitment(&public_key).is_some());
                to_json_binary(&can_reveal.unwrap_or(false))?
            }
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
            QueryMsg::GetDataRequestsByStatus { .. } if contract_paused => {
                let response = GetDataRequestsByStatusResponse {
                    is_paused:     contract_paused,
                    data_requests: Vec::with_capacity(0),
                };
                to_json_binary(&response)?
            }
            QueryMsg::GetDataRequestsByStatus { status, offset, limit } => {
                let response = GetDataRequestsByStatusResponse {
                    is_paused:     contract_paused,
                    data_requests: state::requests_by_status(deps.storage, &status, offset, limit)?,
                };
                to_json_binary(&response)?
            }
        };

        Ok(binary)
    }
}
