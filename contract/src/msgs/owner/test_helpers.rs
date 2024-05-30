use cosmwasm_std::{from_json, testing::mock_env};

use super::{execute::*, *};
use crate::contract::{execute, query};

pub fn transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> Result<Response, ContractError> {
    let msg = transfer_ownership::Execute { new_owner };

    execute(deps, mock_env(), info, msg.into())
}
pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = accept_ownership::Execute {};

    execute(deps, mock_env(), info, msg.into())
}

pub fn add_to_allowlist(deps: DepsMut, info: MessageInfo, public_key: PublicKey) -> Result<Response, ContractError> {
    let msg = add_to_allowlist::Execute { public_key };

    execute(deps, mock_env(), info, msg.into())
}

pub fn remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    public_key: PublicKey,
) -> Result<Response, ContractError> {
    let msg = remove_from_allowlist::Execute { public_key };

    execute(deps, mock_env(), info, msg.into())
}

pub fn get_owner(deps: DepsMut) -> Addr {
    let res = query(deps.as_ref(), mock_env(), query::QueryMsg::GetOwner {}.into()).unwrap();
    from_json(res).unwrap()
}

pub fn get_pending_owner(deps: DepsMut) -> Option<Addr> {
    let res = query(deps.as_ref(), mock_env(), query::QueryMsg::GetPendingOwner {}.into()).unwrap();
    from_json(res).unwrap()
}
