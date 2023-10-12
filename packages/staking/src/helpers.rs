use cosmwasm_std::{from_binary, Addr, DepsMut, MessageInfo, Response};

use crate::contract::{execute, instantiate, query, sudo};
use common::{
    error::ContractError,
    msg::{
        GetDataRequestExecutorResponse, InstantiateMsg, StakingExecuteMsg, StakingQueryMsg, SudoMsg,
    },
    state::Config,
};

use cosmwasm_std::testing::mock_env;

pub fn instantiate_staking_contract(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        proxy: "proxy".to_string(),
    };
    instantiate(deps, mock_env(), info, msg)
}

pub fn helper_register_executor(
    deps: DepsMut,
    info: MessageInfo,
    p2p_multi_address: Option<String>,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let msg = StakingExecuteMsg::RegisterDataRequestExecutor {
        p2p_multi_address,
        sender,
    };
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
    let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
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

pub fn helper_set_config(deps: DepsMut, config: Config) -> Result<Response, ContractError> {
    let msg = SudoMsg::SetConfig { config };
    sudo(deps, mock_env(), msg)
}
