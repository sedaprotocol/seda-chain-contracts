use cosmwasm_std::{from_json, testing::mock_env, Addr, DepsMut, MessageInfo, Response};

use super::TestExecutor;
use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    msgs::{
        staking::{Staker, StakingConfig},
        InstantiateMsg,
        OwnerExecuteMsg,
        OwnerQueryMsg,
        StakingExecuteMsg,
        StakingQueryMsg,
    },
    types::{Secp256k1PublicKey, SimpleHash},
};

pub fn instantiate_staking_contract(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        owner: "owner".to_string(),
    };
    instantiate(deps, mock_env(), info, msg)
}

pub fn reg_and_stake(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    let signature = if let Some(m) = memo.as_ref() {
        exec.sign(["register_and_stake".as_bytes(), &m.simple_hash()])
    } else {
        exec.sign(["register_and_stake".as_bytes()])
    };
    let msg = StakingExecuteMsg::RegisterAndStake { proof: signature, memo };

    execute(deps, mock_env(), info, msg.into())
}

pub fn transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::TransferOwnership { new_owner };

    execute(deps, mock_env(), info, msg.into())
}
pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::AcceptOwnership {};

    execute(deps, mock_env(), info, msg.into())
}
pub fn unregister(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let signature = exec.sign(["unregister".as_bytes()]);
    let msg = StakingExecuteMsg::Unregister { signature };

    execute(deps, mock_env(), info, msg.into())
}

pub fn get_staker(deps: DepsMut, executor: Secp256k1PublicKey) -> Option<Staker> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        StakingQueryMsg::GetStaker { executor }.into(),
    )
    .unwrap();
    let value: Option<Staker> = from_json(res).unwrap();

    value
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
pub fn increase_stake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let signature = exec.sign(["increase_stake".as_bytes()]);
    let msg = StakingExecuteMsg::IncreaseStake { signature };

    execute(deps, mock_env(), info, msg.into())
}

pub fn unstake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor, amount: u128) -> Result<Response, ContractError> {
    let signature = exec.sign(["unstake".as_bytes(), &amount.to_be_bytes()]);
    let msg = StakingExecuteMsg::Unstake { signature, amount };

    execute(deps, mock_env(), info, msg.into())
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    amount: u128,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["withdraw".as_bytes(), &amount.to_be_bytes()]);
    let msg = StakingExecuteMsg::Withdraw { signature, amount };

    execute(deps, mock_env(), info, msg.into())
}

pub fn set_staking_config(deps: DepsMut, info: MessageInfo, config: StakingConfig) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::SetStakingConfig { config };

    execute(deps, mock_env(), info, msg.into())
}

pub fn add_to_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::AddToAllowlist { pub_key };

    execute(deps, mock_env(), info, msg.into())
}

pub fn remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = OwnerExecuteMsg::RemoveFromAllowlist { pub_key };

    execute(deps, mock_env(), info, msg.into())
}
