use common::msg::{GetOwnerResponse, GetPendingOwnerResponse};
use cosmwasm_std::{from_json, Addr, DepsMut, MessageInfo, Response};

use crate::contract::{execute, instantiate, query};
use common::state::StakingConfig;
use common::{
    error::ContractError,
    msg::{GetDataRequestExecutorResponse, InstantiateMsg, StakingExecuteMsg, StakingQueryMsg},
};

use cosmwasm_std::testing::mock_env;

pub fn instantiate_staking_contract(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
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
    memo: Option<String>,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::RegisterDataRequestExecutor { memo, sender };
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
pub fn helper_accept_ownership(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::AcceptOwnership {};
    execute(deps, mock_env(), info, msg)
}
pub fn helper_unregister_executor(
    deps: DepsMut,
    info: MessageInfo,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::UnregisterDataRequestExecutor { sender };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_get_executor(deps: DepsMut, executor: Addr) -> GetDataRequestExecutorResponse {
    let res = query(
        deps.as_ref(),
        mock_env(),
        StakingQueryMsg::GetDataRequestExecutor { executor },
    )
    .unwrap();
    let value: GetDataRequestExecutorResponse = from_json(&res).unwrap();
    value
}
pub fn helper_get_owner(deps: DepsMut) -> GetOwnerResponse {
    let res = query(deps.as_ref(), mock_env(), StakingQueryMsg::GetOwner {}).unwrap();
    let value: GetOwnerResponse = from_json(&res).unwrap();
    value
}

pub fn helper_get_pending_owner(deps: DepsMut) -> GetPendingOwnerResponse {
    let res = query(
        deps.as_ref(),
        mock_env(),
        StakingQueryMsg::GetPendingOwner {},
    )
    .unwrap();
    let value: GetPendingOwnerResponse = from_json(&res).unwrap();
    value
}
pub fn helper_deposit_and_stake(
    deps: DepsMut,
    info: MessageInfo,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::DepositAndStake { sender };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_unstake(
    deps: DepsMut,
    info: MessageInfo,
    amount: u128,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::Unstake { amount, sender };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    amount: u128,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::Withdraw { amount, sender };
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
    address: String,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::AddToAllowlist {
        address: Addr::unchecked(address),
        sender,
    };
    execute(deps, mock_env(), info, msg)
}

pub fn helper_remove_from_allowlist(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::RemoveFromAllowlist {
        address: Addr::unchecked(address),
        sender,
    };
    execute(deps, mock_env(), info, msg)
}
