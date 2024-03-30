#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::state::{CONFIG, DATA_REQUEST_EXECUTORS, TOKEN};
use crate::utils::get_attached_funds;

use common::msg::GetDataRequestExecutorResponse;
use common::state::DataRequestExecutor;

pub mod data_request_executors {
    use common::{error::ContractError, msg::IsDataRequestExecutorEligibleResponse};
    use cosmwasm_std::{Addr, Event};

    use crate::{
        contract::CONTRACT_VERSION,
        state::{ALLOWLIST, ELIGIBLE_DATA_REQUEST_EXECUTORS},
        utils::{apply_validator_eligibility, caller_is_proxy},
    };

    use super::*;

    /// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        memo: Option<String>,
        sender: String,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        caller_is_proxy(&deps, info.sender)?;

        // if allowlist is on, check if the sender is in the allowlist
        let allowlist_enabled = CONFIG.load(deps.storage)?.allowlist_enabled;
        if allowlist_enabled {
            let is_allowed = ALLOWLIST.may_load(deps.storage, sender.clone())?;
            if is_allowed.is_none() {
                return Err(ContractError::NotOnAllowlist);
            }
        }

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        let minimum_stake_to_register = CONFIG.load(deps.storage)?.minimum_stake_to_register;
        if amount < minimum_stake_to_register {
            return Err(ContractError::InsufficientFunds(
                minimum_stake_to_register,
                amount,
            ));
        }

        let executor = DataRequestExecutor {
            memo: memo.clone(),
            tokens_staked: amount,
            tokens_pending_withdrawal: 0,
        };
        DATA_REQUEST_EXECUTORS.save(deps.storage, sender.clone(), &executor)?;

        apply_validator_eligibility(deps, sender.clone(), amount)?;

        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_event(Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", sender.as_ref()),
                ("memo", &memo.unwrap_or_default()),
                ("tokens_staked", &amount.to_string()),
                ("tokens_pending_withdrawal", "0"),
            ])))
    }

    /// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
    pub fn unregister_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        sender: String,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        caller_is_proxy(&deps, info.sender)?;

        // require that the executor has no staked or tokens pending withdrawal
        let executor = DATA_REQUEST_EXECUTORS.load(deps.storage, sender.clone())?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        DATA_REQUEST_EXECUTORS.remove(deps.storage, sender.clone());

        Ok(Response::new()
            .add_attribute("action", "unregister_data_request_executor")
            .add_event(
                Event::new("seda-unregister-data-request-executor")
                    .add_attributes([("version", CONTRACT_VERSION), ("executor", sender.as_ref())]),
            ))
    }

    /// Returns a data request executor from the inactive executors with the given address, if it exists.
    pub fn get_data_request_executor(
        deps: Deps,
        executor: Addr,
    ) -> StdResult<GetDataRequestExecutorResponse> {
        let executor = DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(GetDataRequestExecutorResponse { value: executor })
    }

    /// Returns whether a data request executor is eligible to participate in the committee.
    pub fn is_data_request_executor_eligible(
        deps: Deps,
        executor: Addr,
    ) -> StdResult<IsDataRequestExecutorEligibleResponse> {
        let executor = ELIGIBLE_DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(IsDataRequestExecutorEligibleResponse {
            value: executor.is_some(),
        })
    }
}
