use super::*;

impl SudoHandler for sudo::RemoveDataRequest {
    /// Removes a data request from the contract
    fn sudo(self, mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let event = remove_request(self, &mut deps, &env)?;

        Ok(Response::new().add_event(event))
    }
}
