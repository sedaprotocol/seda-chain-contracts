use std::collections::HashSet;

use super::*;

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(DataRequest)]
    GetDataRequest { dr_id: Hash },
    #[returns(Option<Hash>)]
    GetDataRequestCommitment { dr_id: Hash, public_key: PublicKey },
    #[returns(HashMap<String, Hash>)]
    GetDataRequestCommitments { dr_id: Hash },
    #[returns(Option<RevealBody>)]
    GetDataRequestReveal { dr_id: Hash, public_key: PublicKey },
    #[returns(HashMap<String, RevealBody>)]
    GetDataRequestReveals { dr_id: Hash },
    #[returns(HashSet<DataRequest>)]
    GetDataRequestbyStatus { status: DataRequestStatus },
    #[returns(DataResult)]
    GetResolvedDataRequest { dr_id: Hash },
}

impl QueryMsg {
    pub fn query(self, deps: Deps, _env: Env) -> StdResult<Binary> {
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
            QueryMsg::GetDataRequestbyStatus { status } => {
                to_json_binary(&state::requests_by_status(deps.storage, &status)?)
            }
            QueryMsg::GetResolvedDataRequest { dr_id } => {
                to_json_binary(&state::load_resolved_req(deps.storage, &dr_id)?)
            }
        }
    }
}

#[cfg(test)]
impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::DataRequest(value)
    }
}
