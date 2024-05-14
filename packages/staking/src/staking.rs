use common::{
    crypto::{hash, recover_pubkey},
    error::ContractError,
    types::{Secpk256k1PublicKey, Signature},
};
use cosmwasm_std::{coins, BankMsg, Event};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::CONTRACT_VERSION,
    state::{DATA_REQUEST_EXECUTORS, TOKEN},
    utils::{get_attached_funds, if_allowlist_enabled, update_dr_elig},
};

/// Deposits and stakes tokens for a data request executor.
pub fn deposit_and_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    signature: Signature,
) -> Result<Response, ContractError> {
    let token = TOKEN.load(deps.storage)?;
    let amount = get_attached_funds(&info.funds, &token)?;

    // TODO: do we even need to verify signature for a deposit?
    // compute message hash
    let message_hash = hash(["deposit_and_stake".as_bytes()]);

    // recover public key from signature
    let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;

    // if allowlist is on, check if the signer is in the allowlist
    if_allowlist_enabled(&deps, &public_key)?;

    // update staked tokens for executor
    let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, &public_key)?;
    executor.tokens_staked += amount;
    DATA_REQUEST_EXECUTORS.save(deps.storage, &public_key, &executor)?;

    update_dr_elig(deps, &public_key, executor.tokens_staked)?;

    Ok(Response::new()
        .add_attribute("action", "deposit_and_stake")
        .add_events([
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
    let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, &public_key)?;
    if amount > executor.tokens_staked {
        return Err(ContractError::InsufficientFunds(executor.tokens_staked, amount));
    }

    // update the executor
    executor.tokens_staked -= amount;
    executor.tokens_pending_withdrawal += amount;
    DATA_REQUEST_EXECUTORS.save(deps.storage, &public_key, &executor)?;

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
    let mut executor = DATA_REQUEST_EXECUTORS.load(deps.storage, &public_key)?;
    if amount > executor.tokens_pending_withdrawal {
        return Err(ContractError::InsufficientFunds(
            executor.tokens_pending_withdrawal,
            amount,
        ));
    }

    // update the executor
    executor.tokens_pending_withdrawal -= amount;
    DATA_REQUEST_EXECUTORS.save(deps.storage, &public_key, &executor)?;

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
