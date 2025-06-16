use execute::commit_result::verify_commit;

use super::{
    msgs::data_requests::{execute::commit_result, query::QueryMsg},
    state::DR_CONFIG,
    *,
};
use crate::{msgs::sorted_set::IndexKey, state::PAUSED};

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
                let can_reveal = dr.map(|dr| dr.base.reveal_started() && dr.base.get_commitment(&public_key).is_some());
                to_json_binary(&can_reveal.unwrap_or(false))?
            }
            QueryMsg::GetDataRequest { dr_id } => {
                let dr_id = &Hash::from_hex_str(&dr_id)?;
                match state::may_load_request(deps.storage, dr_id)? {
                    Some(dr) => to_json_binary(&DataRequestResponse {
                        reveals: state::get_reveals(deps.storage, dr_id)?,
                        base:    dr.base,
                    })?,
                    None => to_json_binary(&None::<DataRequestResponse>)?,
                }
            }
            QueryMsg::GetDataRequestCommitment { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.as_ref().map(|dr| dr.base.get_commitment(&public_key)))?
            }
            QueryMsg::GetDataRequestCommitments { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let commitments = dr.map(|dr| dr.base.commits).unwrap_or_default();
                to_json_binary(&commitments)?
            }
            QueryMsg::GetDataRequestReveal { dr_id, public_key } => {
                let dr_id = &Hash::from_hex_str(&dr_id)?;
                if (state::may_load_request(deps.storage, dr_id)?).is_some() {
                    if let Some(reveal) = state::get_reveal(deps.storage, dr_id, &public_key)? {
                        to_json_binary(&reveal)?;
                    }
                }
                to_json_binary(&None::<RevealBody>)?
            }
            QueryMsg::GetDataRequestReveals { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let reveals = dr.map(|dr| dr.reveals).unwrap_or_default();
                to_json_binary(&reveals)?
            }
            QueryMsg::GetDataRequestsByStatus {
                status,
                last_seen_index,
                limit,
            } => {
                let (data_requests, new_last_seen_index, total) = state::requests_by_status(
                    deps.storage,
                    &status,
                    last_seen_index.map(IndexKey::try_from).transpose()?,
                    limit,
                )?;

                let response = GetDataRequestsByStatusResponse {
                    is_paused: contract_paused,
                    data_requests,
                    last_seen_index: new_last_seen_index.map(Into::into),
                    total,
                };
                to_json_binary(&response)?
            }
            QueryMsg::GetDrConfig {} => {
                let config = DR_CONFIG.load(deps.storage)?;
                to_json_binary(&config)?
            }
        };

        Ok(binary)
    }
}
