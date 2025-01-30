use super::{
    msgs::data_requests::execute::{self, ExecuteMsg},
    *,
};
use crate::state::PAUSED;

pub(in crate::msgs::data_requests) mod commit_result;
pub(crate) mod dr_events;
pub(in crate::msgs::data_requests) mod post_request;
pub(in crate::msgs::data_requests) mod reveal_result;
pub(in crate::msgs::data_requests) mod set_timeout_config;

impl ExecuteHandler for ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // setting the timeout config is an owner operation and should not be paused
        if PAUSED.load(deps.storage)? && !matches!(self, ExecuteMsg::SetTimeoutConfig(_)) {
            return Err(ContractError::ContractPaused(
                "data request execute messages".to_string(),
            ));
        }

        match self {
            ExecuteMsg::CommitDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::PostDataRequest(msg) => msg.execute(deps, env, info),
            ExecuteMsg::RevealDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::SetTimeoutConfig(msg) => msg.execute(deps, env, info),
        }
    }
}
