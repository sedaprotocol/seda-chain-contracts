use super::{
    msgs::data_requests::sudo::{self, SudoMsg},
    *,
};
pub(in crate::msgs::data_requests) mod post_result;

impl SudoHandler for SudoMsg {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::PostDataResult(sudo) => sudo.sudo(deps, env),
        }
    }
}
