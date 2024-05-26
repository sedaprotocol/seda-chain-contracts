use cosmwasm_std::{from_json, testing::mock_env, Addr, DepsMut, MessageInfo, Response};

use super::TestExecutor;
use crate::{
    contract::{execute, instantiate, query},
    crypto::hash,
    error::ContractError,
    msgs::{
        staking::{RegisterAndStake, Staker, StakingConfig},
        InstantiateMsg,
        OwnerExecuteMsg,
        OwnerQueryMsg,
        StakingExecuteMsg,
        StakingQueryMsg,
    },
    types::{Hasher, PublicKey},
};

pub fn transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::TransferOwnership { new_owner };

    execute(deps, mock_env(), info, msg.into())
}
pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::AcceptOwnership {};

    execute(deps, mock_env(), info, msg.into())
}

pub fn add_to_allowlist(deps: DepsMut, info: MessageInfo, pub_key: PublicKey) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::AddToAllowlist { pub_key };

    execute(deps, mock_env(), info, msg.into())
}

pub fn remove_from_allowlist(deps: DepsMut, info: MessageInfo, pub_key: PublicKey) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::RemoveFromAllowlist { pub_key };

    execute(deps, mock_env(), info, msg.into())
}

pub fn get_owner(deps: DepsMut) -> Addr {
    let res = query(deps.as_ref(), mock_env(), OwnerQueryMsg::GetOwner {}.into()).unwrap();
    let value = from_json(res).unwrap();

    value
}

pub fn get_pending_owner(deps: DepsMut) -> Option<Addr> {
    let res = query(deps.as_ref(), mock_env(), OwnerQueryMsg::GetPendingOwner {}.into()).unwrap();
    let value = from_json(res).unwrap();

    value
}
