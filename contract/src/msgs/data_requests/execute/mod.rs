use super::{
    msgs::data_requests::execute::{self, ExecuteMsg},
    *,
};

pub(in crate::msgs::data_requests) mod commit_result;
pub(in crate::msgs::data_requests) mod post_request;
pub(in crate::msgs::data_requests) mod reveal_result;

impl ExecuteHandler for ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::CommitDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::PostDataRequest(msg) => msg.execute(deps, env, info),
            ExecuteMsg::RevealDataResult(msg) => msg.execute(deps, env, info),
        }
    }
}
