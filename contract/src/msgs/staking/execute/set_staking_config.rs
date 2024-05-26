use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};

use super::{state::CONFIG, StakingConfig};
use crate::{contract::CONTRACT_VERSION, error::ContractError, msgs::owner::state::OWNER};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::staking) config: StakingConfig,
}

impl Execute {
    /// Set staking config
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        CONFIG.save(deps.storage, &self.config)?;

        Ok(Response::new()
            .add_attribute("action", "set-staking-config")
            .add_events([Event::new("set-staking-config").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                (
                    "minimum_stake_for_committee_eligibility",
                    self.config.minimum_stake_for_committee_eligibility.to_string(),
                ),
                (
                    "minimum_stake_to_register",
                    self.config.minimum_stake_to_register.to_string(),
                ),
                ("allowlist_enabled", self.config.allowlist_enabled.to_string()),
            ])]))
    }
}

impl From<StakingConfig> for crate::msgs::ExecuteMsg {
    fn from(config: StakingConfig) -> Self {
        super::ExecuteMsg::SetStakingConfig(Execute { config }).into()
    }
}
