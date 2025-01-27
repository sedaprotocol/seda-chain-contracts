use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use seda_common::msgs::owner::execute;

use crate::{
    contract::CONTRACT_VERSION,
    error::ContractError,
    msgs::{owner::state::OWNER, ExecuteHandler},
    state::PAUSED,
};

impl ExecuteHandler for execute::unpause::Execute {
    fn execute(self, deps: DepsMut, _: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        let paused = PAUSED.load(deps.storage)?;
        if !paused {
            return Err(ContractError::ContractNotPaused);
        }

        PAUSED.save(deps.storage, &false)?;

        Ok(Response::new().add_events([Event::new("seda-pause-contract")
            .add_attributes([("version", CONTRACT_VERSION.to_string()), ("paused", false.to_string())])]))
    }
}
