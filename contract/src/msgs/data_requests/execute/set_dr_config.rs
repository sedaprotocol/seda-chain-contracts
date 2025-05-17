use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use seda_common::msgs::data_requests::DrConfig;

use super::{dr_events::create_dr_config_event, owner::state::OWNER, state::DR_CONFIG, ContractError, ExecuteHandler};

impl ExecuteHandler for DrConfig {
    /// Set staking config
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        DR_CONFIG.save(deps.storage, &self)?;

        Ok(Response::new()
            .add_attribute("action", "set-timeout-config")
            .add_event(create_dr_config_event(self)))
    }
}
