use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use seda_common::msgs::data_requests::TimeoutConfig;

use super::{
    dr_events::create_timeout_config_event,
    owner::state::OWNER,
    state::TIMEOUT_CONFIG,
    ContractError,
    ExecuteHandler,
};

impl ExecuteHandler for TimeoutConfig {
    /// Set staking config
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        TIMEOUT_CONFIG.save(deps.storage, &self)?;

        Ok(Response::new()
            .add_attribute("action", "set-timeout-config")
            .add_event(create_timeout_config_event(self)))
    }
}
