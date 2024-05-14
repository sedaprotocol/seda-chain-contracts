use common::{
    error::ContractError,
    msg::{
        GetDataRequestExecutorResponse,
        GetOwnerResponse,
        GetPendingOwnerResponse,
        InstantiateMsg,
        StakingExecuteMsg,
        StakingQueryMsg,
    },
    state::StakingConfig,
    test_utils::TestExecutor,
    types::{Secpk256k1PublicKey, SimpleHash},
};
use cosmwasm_std::{from_json, testing::mock_env, DepsMut, MessageInfo, Response};

use crate::contract::{execute, instantiate, query};

pub fn instantiate_staking_contract(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        proxy: "proxy".to_string(),
        owner: "owner".to_string(),
    };
    instantiate(deps, mock_env(), info, msg)
}

pub fn helper_register_executor(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    let signature = if let Some(m) = memo.as_ref() {
        exec.sign([
            "register_data_request_executor".as_bytes().to_vec(),
            m.simple_hash().to_vec(),
        ])
    } else {
        exec.sign(["register_data_request_executor".as_bytes().to_vec()])
    };
    let msg = StakingExecuteMsg::RegisterDataRequestExecutor { signature, memo };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::TransferOwnership { new_owner };
    execute(deps, mock_env(), info, msg)
}
pub fn helper_accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::AcceptOwnership {};
    execute(deps, mock_env(), info, msg)
}
pub fn helper_unregister_executor(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["unregister_data_request_executor".as_bytes().to_vec()]);
    let msg = StakingExecuteMsg::UnregisterDataRequestExecutor { signature };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_get_executor(deps: DepsMut, executor: Secpk256k1PublicKey) -> GetDataRequestExecutorResponse {
    let res = query(
        deps.as_ref(),
        mock_env(),
        StakingQueryMsg::GetDataRequestExecutor { executor },
    )
    .unwrap();
    let value: GetDataRequestExecutorResponse = from_json(res).unwrap();
    value
}
pub fn helper_get_owner(deps: DepsMut) -> GetOwnerResponse {
    let res = query(deps.as_ref(), mock_env(), StakingQueryMsg::GetOwner {}).unwrap();
    let value: GetOwnerResponse = from_json(res).unwrap();
    value
}

pub fn helper_get_pending_owner(deps: DepsMut) -> GetPendingOwnerResponse {
    let res = query(deps.as_ref(), mock_env(), StakingQueryMsg::GetPendingOwner {}).unwrap();
    let value: GetPendingOwnerResponse = from_json(res).unwrap();
    value
}
pub fn helper_deposit_and_stake(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["deposit_and_stake".as_bytes().to_vec()]);
    let msg = StakingExecuteMsg::DepositAndStake { signature };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_unstake(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    amount: u128,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["unstake".as_bytes().to_vec(), amount.to_be_bytes().to_vec()]);
    let msg = StakingExecuteMsg::Unstake { signature, amount };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    amount: u128,
) -> Result<Response, ContractError> {
    let signature = exec.sign(["withdraw".as_bytes().to_vec(), amount.to_be_bytes().to_vec()]);
    let msg = StakingExecuteMsg::Withdraw { signature, amount };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_set_staking_config(
    deps: DepsMut,
    info: MessageInfo,
    config: StakingConfig,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::SetStakingConfig { config };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_add_to_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secpk256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::AddToAllowlist { pub_key };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    pub_key: Secpk256k1PublicKey,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::RemoveFromAllowlist { pub_key };
    execute(deps, mock_env(), info, msg)
}
