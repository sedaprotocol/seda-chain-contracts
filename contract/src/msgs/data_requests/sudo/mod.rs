use super::*;

pub(in crate::msgs::data_requests) mod post_result;

#[cw_serde]
pub enum SudoMsg {
    PostDataResult(post_result::Sudo),
}

impl SudoMsg {
    pub fn execute(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::PostDataResult(sudo) => sudo.execute(deps, env),
        }
    }
}

#[cfg(test)]
impl From<SudoMsg> for super::SudoMsg {
    fn from(value: SudoMsg) -> Self {
        Self::DataRequest(value)
    }
}
