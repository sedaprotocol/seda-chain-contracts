#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::state::DATA_REQUEST_EXECUTORS;

use crate::state::TOKEN;
use crate::utils::{get_attached_funds, validate_sender};

#[allow(clippy::module_inception)]
pub mod staking {
    use cosmwasm_std::{coins, BankMsg, Event};

    use crate::{contract::CONTRACT_VERSION, utils::apply_validator_eligibility};

    use super::*;

    /// Deposits and stakes tokens for a data request executor.
    pub fn deposit_and_stake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        // update staked tokens for executor
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, sender.clone())?;
        executor.tokens_staked += amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, sender.clone(), &executor)?;

        apply_validator_eligibility(deps, sender.clone(), executor.tokens_staked)?;

        Ok(Response::new()
            .add_attribute("action", "deposit_and_stake")
            .add_events(vec![
                Event::new("seda-data-request-executor").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    (
                        "p2p_multi_address",
                        &executor.p2p_multi_address.unwrap_or_default(),
                    ),
                    ("tokens_staked", &executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        &executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-deposit-and-stake").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    ("amount_deposited", &amount.to_string()),
                ]),
            ]))
    }

    /// Unstakes tokens to be withdrawn after a delay.
    pub fn unstake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        amount: u128,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // error if amount is greater than staked tokens
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, sender.clone())?;
        if amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_staked,
                amount,
            ));
        }

        // update the executor
        executor.tokens_staked -= amount;
        executor.tokens_pending_withdrawal += amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, sender.clone(), &executor)?;

        apply_validator_eligibility(deps, sender.clone(), executor.tokens_staked)?;

        // TODO: emit when pending tokens can be withdrawn
        Ok(Response::new()
            .add_attribute("action", "unstake")
            .add_events(vec![
                Event::new("seda-data-request-executor").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    (
                        "p2p_multi_address",
                        &executor.p2p_multi_address.unwrap_or_default(),
                    ),
                    ("tokens_staked", &executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        &executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-unstake").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    ("amount_unstaked", &amount.to_string()),
                ]),
            ]))
    }

    /// Sends tokens back to the executor that are marked as pending withdrawal.
    pub fn withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        amount: u128,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, sender.clone())?;
        if amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                amount,
            ));
        }

        // update the executor
        executor.tokens_pending_withdrawal -= amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, sender.clone(), &executor)?;

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: sender.to_string(),
            amount: coins(amount, token),
        };

        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_events(vec![
                Event::new("seda-data-request-executor").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    (
                        "p2p_multi_address",
                        &executor.p2p_multi_address.unwrap_or_default(),
                    ),
                    ("tokens_staked", &executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        &executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-withdraw").add_attributes(vec![
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    ("amount_withdrawn", &amount.to_string()),
                ]),
            ]))
    }
}

#[cfg(test)]
mod staking_tests {

    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::msg::InstantiateMsg;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use common::msg::GetDataRequestExecutorResponse;
    use common::msg::StakingExecuteMsg as ExecuteMsg;
    use common::msg::StakingQueryMsg as QueryMsg;
    use common::state::DataRequestExecutor;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};
    #[test]
    fn deposit_stake_withdraw() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(0, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // cant register without depositing tokens
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: None,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

        // register a data request executor
        let info = mock_info("anyone", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: None,
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 1,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor stakes 2 more tokens
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::DepositAndStake { sender: None };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 3
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 3,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor unstakes 1
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake {
            amount: 1,
            sender: None,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1 and pending 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 1
                })
            }
        );

        // the data request executor withdraws 1
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Withdraw {
            amount: 1,
            sender: None,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // data request executor's stake should be 1 and pending 0
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // unstake 2 more
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake {
            amount: 2,
            sender: None,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // assert executer is no longer eligible for committe inclusion
        let executor_is_eligible =
            ELIGIBLE_DATA_REQUEST_EXECUTORS.has(&deps.storage, info.sender.clone());
        assert!(!executor_is_eligible);
    }
}
