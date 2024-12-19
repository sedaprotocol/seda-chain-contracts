use cosmwasm_std::{DepsMut, Env, Response};
use seda_common::msgs::data_requests::sudo::SudoMsg;

use super::{ContractError, SudoHandler};

pub(in crate::msgs::data_requests) mod expire_data_requests;
pub(in crate::msgs::data_requests) mod remove_requests;

impl SudoHandler for SudoMsg {
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        match self {
            SudoMsg::RemoveDataRequests(sudo) => sudo.sudo(deps, env),
            SudoMsg::ExpireDataRequests(sudo) => sudo.sudo(deps, env),
        }
    }
}
