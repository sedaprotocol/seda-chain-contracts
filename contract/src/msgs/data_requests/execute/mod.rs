use super::*;

mod post;

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest(post::Execute),
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::PostDataRequest(msg) => msg.execute(deps, info),
        }
    }
}

#[cfg(test)]
impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::DataRequest(value)
    }
}
