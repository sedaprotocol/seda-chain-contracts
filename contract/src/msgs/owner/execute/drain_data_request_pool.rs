use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use seda_common::msgs::owner::execute;

use crate::{
    contract::CONTRACT_VERSION,
    error::ContractError,
    msgs::{owner::state::OWNER, ExecuteHandler},
    state::{DR_POOL_DRAIN_TARGET, PAUSED},
};

impl ExecuteHandler for execute::drain_data_request_pool::Execute {
    fn execute(self, deps: DepsMut, _: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::NotOwner);
        }

        let paused = PAUSED.load(deps.storage)?;
        if paused {
            return Err(ContractError::ContractPaused("pause".to_string()));
        }

        DR_POOL_DRAIN_TARGET.save(deps.storage, &self.target_height)?;

        Ok(Response::new()
            .add_attribute("action", "drain_data_request_pool")
            .add_event(Event::new("seda-contract").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("target_height", self.target_height.to_string()),
            ])))
    }
}
