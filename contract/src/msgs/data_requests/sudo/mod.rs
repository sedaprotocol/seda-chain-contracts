use super::{
    msgs::data_requests::sudo::{self, SudoMsg},
    *,
};
pub(in crate::msgs::data_requests) mod expire_data_requests;
pub(in crate::msgs::data_requests) mod remove_request;
pub(in crate::msgs::data_requests) mod remove_requests;

fn remove_request(request: sudo::RemoveDataRequest, deps: &mut DepsMut, env: &Env) -> Result<Event, ContractError> {
    // find the data request from the committed pool (if it exists, otherwise error)
    let dr_id = Hash::from_hex_str(&request.dr_id)?;
    state::load_request(deps.storage, &dr_id)?;

    let block_height: u64 = env.block.height;

    let event =
        Event::new("remove-dr").add_attributes([("dr_id", request.dr_id), ("block_height", block_height.to_string())]);

    state::remove_request(deps.storage, dr_id)?;

    Ok(event)
}

impl SudoHandler for SudoMsg {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::RemoveDataRequest(sudo) => sudo.sudo(deps, env),
            SudoMsg::RemoveDataRequests(sudo) => sudo.sudo(deps, env),
            SudoMsg::ExpireDataRequests(sudo) => sudo.sudo(deps, env),
        }
    }
}
