use super::*;
use crate::error::ContractError;

mod increase_and_stake;
mod register_and_stake;
mod set_staking_config;
mod unregister;
mod unstake;
mod withdraw;

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAndStake(register_and_stake::Execute),
    Unregister(unregister::Execute),
    IncreaseStake(increase_and_stake::Execute),
    Unstake(unstake::Execute),
    Withdraw(withdraw::Execute),
    SetStakingConfig(set_staking_config::Execute),
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

impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Staking(value)
    }
}
