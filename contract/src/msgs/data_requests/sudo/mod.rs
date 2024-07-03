use super::{
    msgs::data_requests::sudo::{self, SudoMsg},
    *,
};
pub(in crate::msgs::data_requests) mod post_result;
pub(in crate::msgs::data_requests) mod post_results;

fn post_result(result: sudo::PostResult, deps: &mut DepsMut, env: &Env) -> Result<Event, ContractError> {
    // find the data request from the committed pool (if it exists, otherwise error)
    let dr_id = Hash::from_hex_str(&result.dr_id)?;
    let dr = state::load_request(deps.storage, &dr_id)?;

    if !dr.is_tallying() {
        return Err(ContractError::NotEnoughReveals);
    }

    let block_height: u64 = env.block.height;

    let event = Event::new("seda-result").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("dr_id", result.dr_id),
        ("block_height", block_height.to_string()),
        ("exit_code", result.exit_code.to_string()),
        ("result", to_json_string(&result.result)?),
        ("payback_address", dr.payback_address.to_base64()),
        ("seda_payload", dr.seda_payload.to_base64()),
    ]);

    state::post_result(deps.storage, &dr_id, &result.result)?;

    let result_id = result.result.try_hash()?;
    Ok(event.add_attribute("result_id", result_id.to_hex()))
}

impl SudoHandler for SudoMsg {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::PostDataResult(sudo) => sudo.sudo(deps, env),
            SudoMsg::PostDataResults(sudo) => sudo.sudo(deps, env),
        }
    }
}
