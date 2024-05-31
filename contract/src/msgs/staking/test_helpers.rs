use cosmwasm_std::{from_json, testing::mock_env};

use super::{execute::*, *};
use crate::{
    contract::{execute, query},
    crypto::hash,
    types::{Hasher, PublicKey},
    TestExecutor,
    TestInfo,
};

impl TestInfo {
    pub fn set_staking_config(&mut self, sender: &TestExecutor, config: StakingConfig) -> Result<(), ContractError> {
        let msg = config.into();
        self.execute(sender, &msg)
    }
}

pub fn reg_and_stake(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    let msg_hash = hash(["register_and_stake".as_bytes(), &memo.hash()]);

    let msg = register_and_stake::Execute {
        public_key: exec.pub_key(),
        proof: exec.prove(&msg_hash),
        memo,
    };

    execute(deps, mock_env(), info, msg.into())
}

pub fn unregister(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let msg_hash = hash(["unregister".as_bytes()]);
    let msg = unregister::Execute {
        public_key: exec.pub_key(),
        proof:      exec.prove(&msg_hash),
    };

    execute(deps, mock_env(), info, msg.into())
}

pub fn get_staker(deps: DepsMut, executor: PublicKey) -> Option<Staker> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetStaker { executor }.into(),
    )
    .unwrap();
    let value: Option<Staker> = from_json(res).unwrap();

    value
}

pub fn increase_stake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor) -> Result<Response, ContractError> {
    let msg_hash = hash(["increase_stake".as_bytes()]);
    let msg = increase_stake::Execute {
        public_key: exec.pub_key(),
        proof:      exec.prove(&msg_hash),
    };

    execute(deps, mock_env(), info, msg.into())
}

pub fn unstake(deps: DepsMut, info: MessageInfo, exec: &TestExecutor, amount: u128) -> Result<Response, ContractError> {
    let msg_hash = hash(["unstake".as_bytes(), &amount.to_be_bytes()]);
    let msg = unstake::Execute {
        public_key: exec.pub_key(),
        proof:      exec.prove(&msg_hash),
        amount:     amount.into(),
    };

    execute(deps, mock_env(), info, msg.into())
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    amount: u128,
) -> Result<Response, ContractError> {
    let msg_hash = hash(["withdraw".as_bytes(), &amount.to_be_bytes()]);
    let msg = withdraw::Execute {
        public_key: exec.pub_key(),
        proof:      exec.prove(&msg_hash),
        amount:     amount.into(),
    };

    execute(deps, mock_env(), info, msg.into())
}
