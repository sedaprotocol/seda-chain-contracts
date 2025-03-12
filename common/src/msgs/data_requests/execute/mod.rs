use super::TimeoutConfig;

pub mod commit_result;
pub mod post_request;
pub mod reveal_result;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum ExecuteMsg {
    CommitDataResult(commit_result::Execute),
    PostDataRequest(post_request::Execute),
    RevealDataResult(reveal_result::Execute),
    SetTimeoutConfig(TimeoutConfig),
}

impl From<ExecuteMsg> for crate::msgs::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::DataRequest(Box::new(value))
    }
}
