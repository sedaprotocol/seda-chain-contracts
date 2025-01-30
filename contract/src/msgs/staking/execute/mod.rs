use super::{
    msgs::staking::execute::{self, ExecuteMsg},
    *,
};
use crate::state::PAUSED;

pub(in crate::msgs::staking) mod set_staking_config;
pub(in crate::msgs::staking) mod stake;
pub(crate) mod staking_events;
pub(in crate::msgs::staking) mod unstake;
pub(in crate::msgs::staking) mod withdraw;

impl ExecuteHandler for ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // setting the staking config is an owner operation and should not be paused
        if PAUSED.load(deps.storage)? && !matches!(self, ExecuteMsg::SetStakingConfig(_)) {
            return Err(ContractError::ContractPaused("staking execute messages".to_string()));
        }

        match self {
            ExecuteMsg::Stake(msg) => ExecuteHandler::execute(msg, deps, env, info),
            ExecuteMsg::Unstake(msg) => ExecuteHandler::execute(msg, deps, env, info),
            ExecuteMsg::Withdraw(msg) => ExecuteHandler::execute(msg, deps, env, info),
            ExecuteMsg::SetStakingConfig(msg) => ExecuteHandler::execute(msg, deps, env, info),
        }
    }
}
