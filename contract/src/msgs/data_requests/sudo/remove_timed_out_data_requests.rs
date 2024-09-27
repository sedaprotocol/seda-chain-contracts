use cosmwasm_std::{to_json_string, DepsMut, Env, Event, Response};

use super::{state, ContractError, CONTRACT_VERSION};

pub fn remove_timed_out_data_requests(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let ids = state::move_timed_out_requests_to_tally(deps.storage, env.block.height)?;

    let event = Event::new("seda-result").add_attributes([
        ("version", CONTRACT_VERSION.to_string()),
        ("current_height", env.block.height.to_string()),
        ("timed_out_drs", to_json_string(&ids)?),
    ]);
    Ok(Response::new().add_event(event))
}
