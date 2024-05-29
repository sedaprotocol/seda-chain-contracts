use super::*;

pub(in crate::msgs::data_requests) mod commit_result;
pub(in crate::msgs::data_requests) mod post_request;
pub(in crate::msgs::data_requests) mod reveal_result;

#[cw_serde]
pub enum ExecuteMsg {
    CommitDataResult(commit_result::Execute),
    PostDataRequest(post_request::Execute),
    RevealDataResult(reveal_result::Execute),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::CommitDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::PostDataRequest(msg) => msg.execute(deps, info),
            ExecuteMsg::RevealDataResult(msg) => msg.execute(deps, env, info),
        }
    }
}

#[cfg(test)]
impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::DataRequest(Box::new(value))
    }
}
