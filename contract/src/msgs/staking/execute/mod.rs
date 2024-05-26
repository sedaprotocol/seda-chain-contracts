use super::*;

pub(in crate::msgs::staking) mod increase_stake;
pub(in crate::msgs::staking) mod register_and_stake;
pub(in crate::msgs::staking) mod set_staking_config;
pub(in crate::msgs::staking) mod unregister;
pub(in crate::msgs::staking) mod unstake;
pub(in crate::msgs::staking) mod withdraw;

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAndStake(register_and_stake::Execute),
    Unregister(unregister::Execute),
    IncreaseStake(increase_stake::Execute),
    Unstake(unstake::Execute),
    Withdraw(withdraw::Execute),
    SetStakingConfig(StakingConfig),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        use self::*;

        match self {
            ExecuteMsg::RegisterAndStake(msg) => msg.execute(deps, info),
            ExecuteMsg::IncreaseStake(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Unstake(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Withdraw(msg) => msg.execute(deps, env, info),
            ExecuteMsg::Unregister(msg) => msg.execute(deps, info),
            ExecuteMsg::SetStakingConfig(msg) => msg.execute(deps, env, info),
        }
    }
}

#[cfg(test)]
impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Staking(value)
    }
}
