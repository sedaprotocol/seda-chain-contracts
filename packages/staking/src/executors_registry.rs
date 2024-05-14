use common::{msg::GetDataRequestExecutorResponse, state::DataRequestExecutor, types::Secpk256k1PublicKey};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::{
    state::{CONFIG, DATA_REQUEST_EXECUTORS, TOKEN},
    utils::get_attached_funds,
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
        state::ELIGIBLE_DATA_REQUEST_EXECUTORS,
        utils::{if_allowlist_enabled, update_dr_elig},
    };

    /// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        signature: Signature,
        memo: Option<String>,
    ) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = if let Some(m) = memo.as_ref() {
            hash(["register_data_request_executor".as_bytes(), &m.simple_hash()])
        } else {
            hash(["register_data_request_executor".as_bytes()])
        };

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // if allowlist is on, check if the signer is in the allowlist
        if_allowlist_enabled(&deps, &public_key)?;

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
        DATA_REQUEST_EXECUTORS.save(deps.storage, &public_key, &executor)?;

        update_dr_elig(deps, &public_key, amount)?;

        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_event(Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", hex::encode(public_key).as_str()),
                ("sender", info.sender.as_ref()),
                ("memo", &memo.unwrap_or_default()),
                ("tokens_staked", &amount.to_string()),
                ("tokens_pending_withdrawal", "0"),
            ])))
    }

    /// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
    pub fn unregister_data_request_executor(
        deps: DepsMut,
        _info: MessageInfo,
        signature: Signature,
    ) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = hash(["unregister_data_request_executor".as_bytes()]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

        // if allowlist is on, check if the signer is in the allowlist
        if_allowlist_enabled(&deps, &public_key)?;

        // require that the executor has no staked or tokens pending withdrawal
        let executor = DATA_REQUEST_EXECUTORS.load(deps.storage, &public_key)?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        DATA_REQUEST_EXECUTORS.remove(deps.storage, &public_key);

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
        let executor = DATA_REQUEST_EXECUTORS.may_load(deps.storage, &executor)?;
        Ok(GetDataRequestExecutorResponse { value: executor })
    }

    /// Returns whether a data request executor is eligible to participate in the committee.
    pub fn is_data_request_executor_eligible(
        deps: Deps,
        executor: Secpk256k1PublicKey,
    ) -> StdResult<IsDataRequestExecutorEligibleResponse> {
        let executor = ELIGIBLE_DATA_REQUEST_EXECUTORS.may_load(deps.storage, &executor)?;
        Ok(IsDataRequestExecutorEligibleResponse {
            value: executor.is_some(),
        })
    }
}
