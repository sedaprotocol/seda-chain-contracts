use super::*;

impl SudoHandler for sudo::post_results::Sudo {
    /// Posts a data request to the pool
    fn sudo(self, mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let mut response = Response::new();
        for event in self
            .results
            .into_iter()
            .map(|result| post_result(result, &mut deps, &env))
        {
            response = response.add_event(event?);
        }
        Ok(response)
    }
}
