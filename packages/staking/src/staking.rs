#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::state::DATA_REQUEST_EXECUTORS;

use crate::state::TOKEN;
use crate::utils::{get_attached_funds, validate_sender};

#[allow(clippy::module_inception)]
pub mod staking {
    use common::error::ContractError;
    use cosmwasm_std::{coins, BankMsg, Event};

    use crate::{
        contract::CONTRACT_VERSION,
        state::{ADMIN, PENDING_ADMIN},
        utils::apply_validator_eligibility,
    };

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
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
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
                Event::new("seda-data-request-executor-deposit-and-stake").add_attributes([
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
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
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
                Event::new("seda-data-request-executor-unstake").add_attributes([
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
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
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
                Event::new("seda-data-request-executor-withdraw").add_attributes([
                    ("version", CONTRACT_VERSION),
                    ("executor", sender.as_ref()),
                    ("amount_withdrawn", &amount.to_string()),
                ]),
            ]))
    }

    /// Transfer contract ownership to a new admin
    pub fn transfer_ownership(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_admin: String,
    ) -> Result<Response, ContractError> {
        if info.sender != ADMIN.load(deps.storage)? {
            return Err(ContractError::NotAdmin);
        }

        PENDING_ADMIN.save(deps.storage, &Some(deps.api.addr_validate(&new_admin)?))?;
        Ok(Response::new()
            .add_attribute("action", "transfer_ownership")
            .add_events([Event::new("seda-transfer-ownership").add_attributes([
                ("version", CONTRACT_VERSION),
                ("sender", info.sender.as_ref()),
                ("new_admin", &new_admin),
            ])]))
    }

    /// Accept contract ownership
    pub fn accept_ownership(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let pending_admin = PENDING_ADMIN.load(deps.storage)?;
        if pending_admin.is_none() {
            return Err(ContractError::NoPendingAdminFound);
        }
        if pending_admin.is_some_and(|owner| owner != info.sender) {
            return Err(ContractError::NotPendingOwner);
        }
        ADMIN.save(deps.storage, &info.sender)?;
        PENDING_ADMIN.save(deps.storage, &None)?;
        Ok(Response::new()
            .add_attribute("action", "accept-ownership")
            .add_events([Event::new("seda-accept-ownership").add_attributes([
                ("version", CONTRACT_VERSION),
                ("new_admin", info.sender.as_ref()),
            ])]))
    }
}

#[cfg(test)]
mod staking_tests {

    use crate::contract::execute;
    use crate::helpers::helper_deposit_and_stake;
    use crate::helpers::helper_get_executor;
    use crate::helpers::helper_register_executor;
    use crate::helpers::helper_unstake;
    use crate::helpers::helper_withdraw;
    use crate::helpers::instantiate_staking_contract;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use common::error::ContractError;
    use common::msg::GetDataRequestExecutorResponse;
    use common::msg::StakingExecuteMsg as ExecuteMsg;
    use common::state::DataRequestExecutor;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};
    #[test]
    fn deposit_stake_withdraw() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(0, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        // cant register without depositing tokens
        let info = mock_info("anyone", &coins(0, "token"));

        let res = helper_register_executor(deps.as_mut(), info, Some("address".to_string()), None);
        assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

        // register a data request executor
        let info = mock_info("anyone", &coins(1, "token"));

        let _res = helper_register_executor(
            deps.as_mut(),
            info.clone(),
            Some("address".to_string()),
            None,
        );
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), Addr::unchecked("anyone"));

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
        let _res = helper_deposit_and_stake(deps.as_mut(), info.clone(), None).unwrap();
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 3
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), Addr::unchecked("anyone"));

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

        let _res = helper_unstake(deps.as_mut(), info.clone(), 1, None);
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1 and pending 1
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), Addr::unchecked("anyone"));

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
        let _res = helper_withdraw(deps.as_mut(), info.clone(), 1, None);

        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // data request executor's stake should be 1 and pending 0
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), Addr::unchecked("anyone"));

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

    #[test]
    #[should_panic(expected = "NoFunds")]
    fn no_funds_provided() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        let msg = ExecuteMsg::DepositAndStake { sender: None };
        let info = mock_info("anyone", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    #[should_panic(expected = "InsufficientFunds")]
    fn insufficient_funds() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        // register a data request executor
        let info = mock_info("anyone", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: None,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // try unstaking more than staked
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake {
            amount: 2,
            sender: None,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }
}
