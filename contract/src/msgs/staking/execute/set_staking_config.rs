use owner::state::OWNER;
use staking_events::create_staking_config_event;

use super::{state::STAKING_CONFIG, *};

impl ExecuteHandler for StakingConfig {
    /// Set staking config
    fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        STAKING_CONFIG.save(deps.storage, &self)?;

        Ok(Response::new()
            .add_attribute("action", "set-staking-config")
            .add_event(create_staking_config_event(self)))
    }
}
