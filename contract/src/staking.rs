use cosmwasm_std::{coins, BankMsg, Deps, Event, StdResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::CONTRACT_VERSION, crypto::{hash, recover_pubkey}, error::ContractError, msg::{GetStaker, IsExecutorEligibleResponse}, state::{Staker, CONFIG, STAKERS, TOKEN}, types::{Secp256k1PublicKey, Signature, SimpleHash}, utils::{get_attached_funds, is_staker_allowed}
};

/// Registers a staker with an optional p2p multi address, requiring a token deposit.
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
    let public_key: Secp256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    is_staker_allowed(&deps, &public_key)?;

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

/// Deposits and stakes tokens for an already existing staker.
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
    let public_key: Secp256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    is_staker_allowed(&deps, &public_key)?;

    // update staked tokens for executor
    let mut executor = STAKERS.load(deps.storage, &public_key)?;
    executor.tokens_staked += amount;
    STAKERS.save(deps.storage, &public_key, &executor)?;

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

/// Unstakes tokens from a given staker, to be withdrawn after a delay.
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
    let public_key: Secp256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // error if amount is greater than staked tokens
    let mut executor = STAKERS.load(deps.storage, &public_key)?;
    if amount > executor.tokens_staked {
        return Err(ContractError::InsufficientFunds(executor.tokens_staked, amount));
    }

    // update the executor
    executor.tokens_staked -= amount;
    executor.tokens_pending_withdrawal += amount;
    STAKERS.save(deps.storage, &public_key, &executor)?;

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

/// Sends tokens back to the sender that are marked as pending withdrawal.
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
    let public_key: Secp256k1PublicKey = recover_pubkey(message_hash, signature)?;

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

/// Unregisters a staker, with the requirement that no tokens are staked or pending withdrawal.
pub fn unregister(deps: DepsMut, _info: MessageInfo, signature: Signature) -> Result<Response, ContractError> {
    // compute message hash
    let message_hash = hash(["unregister".as_bytes()]);

    // recover public key from signature
    let public_key: Secp256k1PublicKey = recover_pubkey(message_hash, signature)?;

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

/// Returns a staker with the given address, if it exists.
pub fn get_staker(deps: Deps, executor: Secp256k1PublicKey) -> StdResult<GetStaker> {
    let executor = STAKERS.may_load(deps.storage, &executor)?;
    Ok(GetStaker { value: executor })
}

// TODO: maybe move this to data-requests contract?
/// Returns whether an executor is eligible to participate in the committee.
pub fn is_executor_eligible(deps: Deps, executor: Secp256k1PublicKey) -> StdResult<IsExecutorEligibleResponse> {
    let executor = STAKERS.may_load(deps.storage, &executor)?;
    let value = match executor {
        Some(staker) => staker.tokens_staked >= CONFIG.load(deps.storage)?.minimum_stake_for_committee_eligibility,
        None => false,
    };

    Ok(IsExecutorEligibleResponse { value })
}
