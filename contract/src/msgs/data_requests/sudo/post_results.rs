use super::*;

impl SudoHandler for sudo::post_results::Sudo {
    /// Posts data results to the contract
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