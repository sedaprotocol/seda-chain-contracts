
use cosmwasm_std::{from_json, testing::mock_env, DepsMut, MessageInfo, Response};

use crate::{contract::{execute, instantiate, query}, error::ContractError, msg::{ExecuteMsg, GetOwnerResponse, GetPendingOwnerResponse, GetStaker, InstantiateMsg, QueryMsg}, state::StakingConfig, types::{Secp256k1PublicKey, SimpleHash}};

use super::test_utils::TestExecutor;

pub fn instantiate_staking_contract(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        // proxy: "proxy".to_string(),
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
    let msg = ExecuteMsg::RegisterAndStake { signature, memo };

    execute(deps, mock_env(), info, msg)
}

pub fn transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::TransferOwnership { new_owner };

    execute(deps, mock_env(), info, msg)
}
pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::AcceptOwnership {};

    execute(deps, mock_env(), info, msg)
}
pub fn unregister(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let signature = exec.sign(["unregister".as_bytes()]);
    let msg = ExecuteMsg::Unregister { signature };

    execute(deps, mock_env(), info, msg)
}

pub fn get_staker(deps: DepsMut, executor: Secp256k1PublicKey) -> GetStaker {
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetStaker { executor }).unwrap();
    let value: GetStaker = from_json(res).unwrap();

    value
}
pub fn get_owner(deps: DepsMut) -> GetOwnerResponse {
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    let value: GetOwnerResponse = from_json(res).unwrap();

    value
}

pub fn get_pending_owner(deps: DepsMut) -> GetPendingOwnerResponse {
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPendingOwner {}).unwrap();
    let value: GetPendingOwnerResponse = from_json(res).unwrap();

    value
}
pub fn increase_stake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let signature = exec.sign(["increase_stake".as_bytes()]);
    let msg = ExecuteMsg::IncreaseStake { signature };

    execute(deps, mock_env(), info, msg)
}

pub fn unstake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor, amount: u128) -> Result<Response, ContractError> {
    let signature = exec.sign(["unstake".as_bytes(), &amount.to_be_bytes()]);
    let msg = ExecuteMsg::Unstake { signature, amount };

    execute(deps, mock_env(), info, msg)
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    amount: u128,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["withdraw".as_bytes(), &amount.to_be_bytes()]);
    let msg = ExecuteMsg::Withdraw { signature, amount };

    execute(deps, mock_env(), info, msg)
}

pub fn set_staking_config(deps: DepsMut, info: MessageInfo, config: StakingConfig) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::SetStakingConfig { config };

    execute(deps, mock_env(), info, msg)
}

pub fn add_to_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::AddToAllowlist { pub_key };

    execute(deps, mock_env(), info, msg)
}

pub fn remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secp256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::RemoveFromAllowlist { pub_key };

    execute(deps, mock_env(), info, msg)
}
