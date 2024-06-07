use super::*;

pub mod commit_result;
pub mod post_request;
pub mod reveal_result;

#[cw_serde]
pub enum ExecuteMsg {
    CommitDataResult(commit_result::Execute),
    PostDataRequest(post_request::Execute),
    RevealDataResult(reveal_result::Execute),
}

impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::DataRequest(Box::new(value))
    }
}
