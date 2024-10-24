use super::*;

impl SudoHandler for sudo::remove_requests::Sudo {
    fn sudo(self, mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let mut response = Response::new();
        for event in self
            .requests
            .into_iter()
            .map(|request| remove_request(request, &mut deps, &env))
        {
            response = response.add_event(event?);
        }
        Ok(response)
    }
}
