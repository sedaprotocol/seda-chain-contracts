use common::{
    crypto::{hash, recover_pubkey},
    error::ContractError,
    msg::{GetStaker, IsDataRequestExecutorEligibleResponse},
    state::Staker,
    types::{Secpk256k1PublicKey, Signature, SimpleHash},
};
use cosmwasm_std::{coins, BankMsg, Deps, Event, StdResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::CONTRACT_VERSION,
    state::{CONFIG, ELIGIBLE_DATA_REQUEST_EXECUTORS, STAKERS, TOKEN},
    utils::{get_attached_funds, if_allowlist_enabled, update_dr_elig},
};

/// Deposits and stakes tokens for a data request executor.
pub fn increase_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    signature: Signature,
) -> Result<Response, ContractError> {
    let token = TOKEN.load(deps.storage)?;
    let amount = get_attached_funds(&info.funds, &token)?;

    // compute message hash
    let message_hash = hash(["increase_stake".as_bytes()]);

    // recover public key from signature
    let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    if_allowlist_enabled(&deps, &public_key)?;

    // update staked tokens for executor
    let mut executor = STAKERS.load(deps.storage, &public_key)?;
    executor.tokens_staked += amount;
    STAKERS.save(deps.storage, &public_key, &executor)?;

    update_dr_elig(deps, &public_key, executor.tokens_staked)?;

    Ok(Response::new().add_attribute("action", "increase-stake").add_events([
        Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION),
            ("executor", &hex::encode(&public_key)),
            ("memo", &executor.memo.unwrap_or_default()),
            ("tokens_staked", &executor.tokens_staked.to_string()),
            (
                "tokens_pending_withdrawal",
                &executor.tokens_pending_withdrawal.to_string(),
            ),
        ]),
        Event::new("seda-data-request-executor-increase-stake").add_attributes([
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
    _info: MessageInfo,
    signature: Signature,
    amount: u128,
) -> Result<Response, ContractError> {
    // compute message hash
    let message_hash = hash(["unstake".as_bytes(), &amount.to_be_bytes()]);

    // recover public key from signature
    let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    if_allowlist_enabled(&deps, &public_key)?;

    // error if amount is greater than staked tokens
    let mut executor = STAKERS.load(deps.storage, &public_key)?;
    if amount > executor.tokens_staked {
        return Err(ContractError::InsufficientFunds(executor.tokens_staked, amount));
    }

    // update the executor
    executor.tokens_staked -= amount;
    executor.tokens_pending_withdrawal += amount;
    STAKERS.save(deps.storage, &public_key, &executor)?;

    update_dr_elig(deps, &public_key, executor.tokens_staked)?;

    // TODO: emit when pending tokens can be withdrawn
    Ok(Response::new().add_attribute("action", "unstake").add_events([
        Event::new("seda-data-request-executor").add_attributes([
            ("version", CONTRACT_VERSION),
            ("executor", &hex::encode(&public_key)),
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
) -> Result<Response, ContractError> {
    // compute message hash
    let message_hash = hash(["withdraw".as_bytes(), &amount.to_be_bytes()]);

    // recover public key from signature
    let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    if_allowlist_enabled(&deps, &public_key)?;

    // TODO: add delay after calling unstake
    let token = TOKEN.load(deps.storage)?;

    // error if amount is greater than pending tokens
    let mut executor = STAKERS.load(deps.storage, &public_key)?;
    if amount > executor.tokens_pending_withdrawal {
        return Err(ContractError::InsufficientFunds(
            executor.tokens_pending_withdrawal,
            amount,
        ));
    }

    // update the executor
    executor.tokens_pending_withdrawal -= amount;
    STAKERS.save(deps.storage, &public_key, &executor)?;

    // send the tokens back to the executor
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount:     coins(amount, token),
    };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw")
        .add_events([
            Event::new("seda-data-request-executor").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", info.sender.as_ref()),
                ("memo", &executor.memo.unwrap_or_default()),
                ("tokens_staked", &executor.tokens_staked.to_string()),
                (
                    "tokens_pending_withdrawal",
                    &executor.tokens_pending_withdrawal.to_string(),
                ),
            ]),
            Event::new("seda-data-request-executor-withdraw").add_attributes([
                ("version", CONTRACT_VERSION),
                ("executor", info.sender.as_ref()),
                ("amount_withdrawn", &amount.to_string()),
            ]),
        ]))
}

/// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
pub fn register_and_stake(
    deps: DepsMut,
    info: MessageInfo,
    signature: Signature,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    // compute message hash
    let message_hash = if let Some(m) = memo.as_ref() {
        hash(["register_and_stake".as_bytes(), &m.simple_hash()])
    } else {
        hash(["register_and_stake".as_bytes()])
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

    let executor = Staker {
        memo:                      memo.clone(),
        tokens_staked:             amount,
        tokens_pending_withdrawal: 0,
    };
    STAKERS.save(deps.storage, &public_key, &executor)?;

    update_dr_elig(deps, &public_key, amount)?;

    Ok(Response::new().add_attribute("action", "register-and-stake").add_event(
        Event::new("seda-register-and-stake").add_attributes([
            ("version", CONTRACT_VERSION),
            ("executor", hex::encode(public_key).as_str()),
            ("sender", info.sender.as_ref()),
            ("memo", &memo.unwrap_or_default()),
            ("tokens_staked", &amount.to_string()),
            ("tokens_pending_withdrawal", "0"),
        ]),
    ))
}

/// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
pub fn unregister(deps: DepsMut, _info: MessageInfo, signature: Signature) -> Result<Response, ContractError> {
    // compute message hash
    let message_hash = hash(["unregister".as_bytes()]);

    // recover public key from signature
    let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    if_allowlist_enabled(&deps, &public_key)?;

    // require that the executor has no staked or tokens pending withdrawal
    let executor = STAKERS.load(deps.storage, &public_key)?;
    if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
        return Err(ContractError::ExecutorHasTokens);
    }

    STAKERS.remove(deps.storage, &public_key);

    Ok(Response::new()
        .add_attribute("action", "unregister")
        .add_event(Event::new("seda-unregister").add_attributes([
            ("version", CONTRACT_VERSION),
            ("executor", hex::encode(public_key).as_str()),
        ])))
}

/// Returns a data request executor from the inactive executors with the given address, if it exists.
pub fn get_staker(deps: Deps, executor: Secpk256k1PublicKey) -> StdResult<GetStaker> {
    let executor = STAKERS.may_load(deps.storage, &executor)?;
    Ok(GetStaker { value: executor })
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
