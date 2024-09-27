use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use seda_common::msgs::data_requests::TimeoutConfig;

use super::{owner::state::OWNER, state::TIMEOUT_CONFIG, ContractError, ExecuteHandler, CONTRACT_VERSION};

impl ExecuteHandler for TimeoutConfig {
    /// Set staking config
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        TIMEOUT_CONFIG.save(deps.storage, &self)?;

        Ok(Response::new()
            .add_attribute("action", "set-timeout-config")
            .add_events([Event::new("set-timeout-config").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("commit_timeout_in_blocks", self.commit_timeout_in_blocks.to_string()),
                ("reveal_timeout_in_blocks", self.reveal_timeout_in_blocks.to_string()),
            ])]))
    }
}
