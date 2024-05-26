use self::staking::owner::state::OWNER;
use super::{state::CONFIG, StakingConfig, *};

impl StakingConfig {
    /// Set staking config
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        CONFIG.save(deps.storage, &self)?;

        Ok(Response::new()
            .add_attribute("action", "set-staking-config")
            .add_events([Event::new("set-staking-config").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                (
                    "minimum_stake_for_committee_eligibility",
                    self.minimum_stake_for_committee_eligibility.to_string(),
                ),
                ("minimum_stake_to_register", self.minimum_stake_to_register.to_string()),
                ("allowlist_enabled", self.allowlist_enabled.to_string()),
            ])]))
    }
}

impl From<StakingConfig> for crate::msgs::ExecuteMsg {
    fn from(config: StakingConfig) -> Self {
        super::ExecuteMsg::SetStakingConfig(config).into()
    }
}
