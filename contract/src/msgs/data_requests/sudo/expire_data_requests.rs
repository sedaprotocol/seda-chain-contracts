use cosmwasm_std::{to_json_string, DepsMut, Env, Response};
use seda_common::msgs::data_requests::sudo::expire_data_requests;

use super::{state, ContractError, SudoHandler};

impl SudoHandler for expire_data_requests::Sudo {
    /// Expires all data requests that have timed out
    /// by moving them from whatever state they are in to the tallying state.
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let ids = state::expire_data_requests(deps.storage, env.block.height)?;

        let response = Response::new().add_attribute("method", "expire-data-requests");

        if ids.is_empty() {
            return Ok(response);
        }

        Ok(response.add_attribute("timed_out_drs", to_json_string(&ids)?))
    }
}
