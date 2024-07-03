use super::*;

impl SudoHandler for sudo::PostResult {
    /// Posts a data request to the pool
    fn sudo(self, mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let event = post_result(self, &mut deps, &env)?;

        Ok(Response::new().add_event(event))
    }
}
