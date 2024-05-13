use common::{msg::GetDataRequestExecutorResponse, state::DataRequestExecutor, types::Secpk256k1PublicKey};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::{
    state::{CONFIG, DATA_REQUEST_EXECUTORS, TOKEN},
    utils::{get_attached_funds, validate_sender},
};

pub mod data_request_executors {
    use common::{
        crypto::{hash, recover_pubkey},
        error::ContractError,
        msg::IsDataRequestExecutorEligibleResponse,
        types::{Signature, SimpleHash},
    };
    use cosmwasm_std::Event;

    use super::*;
    use crate::{
        contract::CONTRACT_VERSION,
        state::{ALLOWLIST, ELIGIBLE_DATA_REQUEST_EXECUTORS},
        utils::apply_validator_eligibility,
    };

    /// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        signature: Signature,
        memo: Option<String>,
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
        let message_hash = if let Some(m) = memo.as_ref() {
            hash([
                "register_data_request_executor".as_bytes(),
                sender.as_bytes(),
                &m.simple_hash(),
            ])
        } else {
            hash(["register_data_request_executor".as_bytes(), sender.as_bytes()])
        };

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

        let minimum_stake_to_register = CONFIG.load(deps.storage)?.minimum_stake_to_register;
        if amount < minimum_stake_to_register {
            return Err(ContractError::InsufficientFunds(minimum_stake_to_register, amount));
        }

        let executor = DataRequestExecutor {
            memo:                      memo.clone(),
            tokens_staked:             amount,
            tokens_pending_withdrawal: 0,
        };
        DATA_REQUEST_EXECUTORS.save(deps.storage, public_key.clone(), &executor)?;

        apply_validator_eligibility(deps, public_key.clone(), amount)?;

        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_event(Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", hex::encode(public_key).as_str()),
                ("sender", sender.as_ref()),
                ("memo", &memo.unwrap_or_default()),
                ("tokens_staked", &amount.to_string()),
                ("tokens_pending_withdrawal", "0"),
            ])))
    }

    /// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
    pub fn unregister_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        signature: Signature,
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
        let message_hash = hash(["unregister_data_request_executor".as_bytes(), sender.as_bytes()]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // require that the executor has no staked or tokens pending withdrawal
        let executor = DATA_REQUEST_EXECUTORS.load(deps.storage, public_key.clone())?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        DATA_REQUEST_EXECUTORS.remove(deps.storage, public_key.clone());

        Ok(Response::new()
            .add_attribute("action", "unregister_data_request_executor")
            .add_event(Event::new("seda-unregister-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", hex::encode(public_key).as_str()),
            ])))
    }

    /// Returns a data request executor from the inactive executors with the given address, if it exists.
    pub fn get_data_request_executor(
        deps: Deps,
        executor: Secpk256k1PublicKey,
    ) -> StdResult<GetDataRequestExecutorResponse> {
        let executor = DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(GetDataRequestExecutorResponse { value: executor })
    }

    /// Returns whether a data request executor is eligible to participate in the committee.
    pub fn is_data_request_executor_eligible(
        deps: Deps,
        executor: Secpk256k1PublicKey,
    ) -> StdResult<IsDataRequestExecutorEligibleResponse> {
        let executor = ELIGIBLE_DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(IsDataRequestExecutorEligibleResponse {
            value: executor.is_some(),
        })
    }
}
