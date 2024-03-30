#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::state::DATA_REQUEST_EXECUTORS;

use crate::state::TOKEN;
use crate::utils::get_attached_funds;

#[allow(clippy::module_inception)]
pub mod staking {
    use common::{error::ContractError, state::StakingConfig};
    use cosmwasm_std::{coins, BankMsg, Event};

    use crate::{
        contract::CONTRACT_VERSION,
        state::{CONFIG, OWNER, PENDING_OWNER},
        utils::{apply_validator_eligibility, caller_is_proxy},
    };

    use super::*;

    /// Deposits and stakes tokens for a data request executor.
    pub fn deposit_and_stake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        sender: String,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        caller_is_proxy(&deps, info.sender)?;

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
                    ("memo", &executor.memo.unwrap_or_default()),
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
        sender: String,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        caller_is_proxy(&deps, info.sender)?;

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
                    ("memo", &executor.memo.unwrap_or_default()),
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
        sender: String,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        caller_is_proxy(&deps, info.sender)?;

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

    /// Transfer contract ownership to a new owner
    pub fn transfer_ownership(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_owner: String,
    ) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }

        PENDING_OWNER.save(deps.storage, &Some(deps.api.addr_validate(&new_owner)?))?;
        Ok(Response::new()
            .add_attribute("action", "transfer_ownership")
            .add_events([Event::new("seda-transfer-ownership").add_attributes([
                ("version", CONTRACT_VERSION),
                ("sender", info.sender.as_ref()),
                ("new_owner", &new_owner),
            ])]))
    }

    /// Accept contract ownership
    pub fn accept_ownership(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let pending_owner = PENDING_OWNER.load(deps.storage)?;
        if pending_owner.is_none() {
            return Err(ContractError::NoPendingOwnerFound);
        }
        if pending_owner.is_some_and(|owner| owner != info.sender) {
            return Err(ContractError::NotPendingOwner);
        }
        OWNER.save(deps.storage, &info.sender)?;
        PENDING_OWNER.save(deps.storage, &None)?;
        Ok(Response::new()
            .add_attribute("action", "accept-ownership")
            .add_events([Event::new("seda-accept-ownership").add_attributes([
                ("version", CONTRACT_VERSION),
                ("new_owner", info.sender.as_ref()),
            ])]))
    }

    /// Set staking config
    pub fn set_staking_config(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        config: StakingConfig,
    ) -> Result<Response, ContractError> {
        if info.sender != OWNER.load(deps.storage)? {
            return Err(ContractError::NotOwner);
        }
        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new()
            .add_attribute("action", "set-staking-config")
            .add_events([Event::new("set-staking-config").add_attributes([
                ("version", CONTRACT_VERSION),
                (
                    "minimum_stake_for_committee_eligibility",
                    &config.minimum_stake_for_committee_eligibility.to_string(),
                ),
                (
                    "minimum_stake_to_register",
                    &config.minimum_stake_to_register.to_string(),
                ),
            ])]))
    }
}
