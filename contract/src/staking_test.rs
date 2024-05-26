use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_env, mock_info},
};

use super::{test_helpers, TestExecutor};
use crate::{
    contract::execute,
    crypto::hash,
    error::ContractError,
    msgs::{staking::Staker, StakingExecuteMsg},
    staking::is_executor_eligible,
};
