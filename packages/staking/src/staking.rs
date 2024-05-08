#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::state::DATA_REQUEST_EXECUTORS;

use crate::state::{ALLOWLIST, TOKEN};
use crate::utils::{get_attached_funds, validate_sender};

#[allow(clippy::module_inception)]
pub mod staking {
    use common::{
        crypto::{hash, recover_pubkey},
        error::ContractError,
        types::{Secpk256k1PublicKey, Signature},
    };
    use cosmwasm_std::{coins, BankMsg, Event};

    use crate::{contract::CONTRACT_VERSION, state::CONFIG, utils::apply_validator_eligibility};

    use super::*;

    /// Deposits and stakes tokens for a data request executor.
    pub fn deposit_and_stake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        signature: Signature,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        // if allowlist is on, check if the sender is in the allowlist
        let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
        if allowlist_enabled {
            let is_allowed = ALLOWLIST.may_load(deps.storage, sender.clone())?;
            if is_allowed.is_none() {
                return Err(ContractError::NotOnAllowlist);
            }
        }

        // TODO: do we even need to verify signature for a deposit?
        // compute message hash
        let message_hash = hash(["deposit_and_stake".as_bytes(), sender.as_bytes()]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // update staked tokens for executor
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, public_key.clone())?;
        executor.tokens_staked += amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, public_key.clone(), &executor)?;

        apply_validator_eligibility(deps, public_key.clone(), executor.tokens_staked)?;

        Ok(Response::new()
            .add_attribute("action", "deposit_and_stake")
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
                    ("version", CONTRACT_VERSION),
                    ("executor", &hex::encode(public_key.clone())),
                    ("memo", &executor.memo.unwrap_or_default()),
                    ("tokens_staked", &executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        &executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-deposit-and-stake").add_attributes([
                    ("version", CONTRACT_VERSION),
                    ("executor", &hex::encode(public_key)),
                    ("amount_deposited", &amount.to_string()),
                ]),
            ]))
    }

    /// Unstakes tokens to be withdrawn after a delay.
    pub fn unstake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        signature: Signature,
        amount: u128,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // if allowlist is on, check if the sender is in the allowlist
        let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
        if allowlist_enabled {
            let is_allowed = ALLOWLIST.may_load(deps.storage, sender.clone())?;
            if is_allowed.is_none() {
                return Err(ContractError::NotOnAllowlist);
            }
        }

        // compute message hash
        let message_hash = hash([
            "unstake".as_bytes(),
            &amount.to_be_bytes(),
            sender.as_bytes(),
        ]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // error if amount is greater than staked tokens
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, public_key.clone())?;
        if amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_staked,
                amount,
            ));
        }

        // update the executor
        executor.tokens_staked -= amount;
        executor.tokens_pending_withdrawal += amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, public_key.clone(), &executor)?;

        apply_validator_eligibility(deps, public_key.clone(), executor.tokens_staked)?;

        // TODO: emit when pending tokens can be withdrawn
        Ok(Response::new()
            .add_attribute("action", "unstake")
            .add_events([
                Event::new("seda-data-request-executor").add_attributes([
                    ("version", CONTRACT_VERSION),
                    ("executor", &hex::encode(public_key.clone())),
                    ("memo", &executor.memo.unwrap_or_default()),
                    ("tokens_staked", &executor.tokens_staked.to_string()),
                    (
                        "tokens_pending_withdrawal",
                        &executor.tokens_pending_withdrawal.to_string(),
                    ),
                ]),
                Event::new("seda-data-request-executor-unstake").add_attributes([
                    ("version", CONTRACT_VERSION),
                    ("executor", &hex::encode(public_key)),
                    ("amount_unstaked", &amount.to_string()),
                ]),
            ]))
    }

    /// Sends tokens back to the executor that are marked as pending withdrawal.
    pub fn withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        signature: Signature,
        amount: u128,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // if allowlist is on, check if the sender is in the allowlist
        let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
        if allowlist_enabled {
            let is_allowed = ALLOWLIST.may_load(deps.storage, sender.clone())?;
            if is_allowed.is_none() {
                return Err(ContractError::NotOnAllowlist);
            }
        }

        // compute message hash
        let message_hash = hash([
            "withdraw".as_bytes(),
            &amount.to_be_bytes(),
            sender.as_bytes(),
        ]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, public_key.clone())?;
        if amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                amount,
            ));
        }

        // update the executor
        executor.tokens_pending_withdrawal -= amount;
        DATA_REQUEST_EXECUTORS.save(deps.storage, public_key.clone(), &executor)?;

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
                    ("memo", &executor.memo.unwrap_or_default()),
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
    use common::test_utils::TestExecutor;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    #[test]
    fn deposit_stake_withdraw() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(0, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        // cant register without depositing tokens
        let info = mock_info("anyone", &coins(0, "token"));
        let exec = TestExecutor::new("anyone");

        let res = helper_register_executor(
            deps.as_mut(),
            info,
            &exec,
            Some("address".to_string()),
            None,
        );
        assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

        // register a data request executor
        let info = mock_info("anyone", &coins(1, "token"));

        let _res = helper_register_executor(
            deps.as_mut(),
            info.clone(),
            &exec,
            Some("address".to_string()),
            None,
        );
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, exec.public_key.clone()) // Convert Addr to Vec<u8>
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), exec.public_key.clone());

        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    memo: Some("address".to_string()),
                    tokens_staked: 1,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor stakes 2 more tokens
        let info = mock_info("anyone", &coins(2, "token"));
        let _res = helper_deposit_and_stake(deps.as_mut(), info.clone(), &exec, None).unwrap();
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, exec.public_key.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 3
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), exec.public_key.clone());

        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    memo: Some("address".to_string()),
                    tokens_staked: 3,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor unstakes 1
        let info = mock_info("anyone", &coins(0, "token"));

        let _res = helper_unstake(deps.as_mut(), info.clone(), &exec, 1, None);
        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, exec.public_key.clone())
            .unwrap();
        assert!(executor_is_eligible);
        // data request executor's stake should be 1 and pending 1
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), exec.public_key.clone());

        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    memo: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 1
                })
            }
        );

        // the data request executor withdraws 1
        let info = mock_info("anyone", &coins(0, "token"));
        let _res = helper_withdraw(deps.as_mut(), info.clone(), &exec, 1, None);

        let executor_is_eligible = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, exec.public_key.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // data request executor's stake should be 1 and pending 0
        let value: GetDataRequestExecutorResponse =
            helper_get_executor(deps.as_mut(), exec.public_key.clone());

        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    memo: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // unstake 2 more
        helper_unstake(deps.as_mut(), info, &exec, 2, None).unwrap();

        // assert executer is no longer eligible for committe inclusion
        let executor_is_eligible =
            ELIGIBLE_DATA_REQUEST_EXECUTORS.has(&deps.storage, exec.public_key.clone());
        assert!(!executor_is_eligible);
    }

    #[test]
    #[should_panic(expected = "NoFunds")]
    fn no_funds_provided() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();
        let exec = TestExecutor::new("anyone");

        let msg = ExecuteMsg::DepositAndStake {
            sender: None,
            signature: exec.sign([
                "register_data_request_executor".as_bytes().to_vec(),
                "anyone".as_bytes().to_vec(),
            ]),
        };
        let info = mock_info("anyone", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    #[should_panic(expected = "InsufficientFunds")]
    fn insufficient_funds() {
        let mut deps = mock_dependencies();

        let info = mock_info("alice", &coins(1, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info.clone()).unwrap();
        let alice = TestExecutor::new("alice");

        // register a data request executor
        helper_register_executor(
            deps.as_mut(),
            info.clone(),
            &alice,
            Some("address".to_string()),
            None,
        )
        .unwrap();

        // try unstaking more than staked
        let info = mock_info("alice", &coins(0, "token"));
        helper_unstake(deps.as_mut(), info.clone(), &alice, 2, None).unwrap();
    }
}
