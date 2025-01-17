use cosmwasm_std::{to_json_string, Addr, BankMsg, Coin, DepsMut, Env, Event, Response, Uint128};
use cw_storage_plus::KeyDeserialize;
use seda_common::{
    msgs::data_requests::sudo::{remove_requests, DistributionKind, DistributionMessages},
    types::Hash,
};
use serde_json::json;

use super::{ContractError, SudoHandler};
use crate::{
    msgs::{
        data_requests::state::{self, DR_ESCROW},
        staking::state::{STAKERS, STAKING_CONFIG},
        PublicKey,
    },
    state::TOKEN,
    types::FromHexStr,
};

fn amount_to_tokens(amount: Uint128, token: &str) -> Coin {
    Coin {
        denom: token.to_string(),
        amount,
    }
}

fn remove_request(
    dr_id_str: String,
    messages: DistributionMessages,
    deps: &mut DepsMut,
    env: &Env,
    token: &str,
) -> Result<(Event, Vec<BankMsg>), ContractError> {
    // find the data request from the committed pool (if it exists, otherwise error)
    let dr_id = Hash::from_hex_str(&dr_id_str)?;
    state::load_request(deps.storage, &dr_id)?;

    let block_height: u64 = env.block.height;

    let mut event =
        Event::new("seda-remove-dr").add_attributes([("dr_id", dr_id_str), ("block_height", block_height.to_string())]);

    let mut dr_escrow = DR_ESCROW.load(deps.storage, &dr_id)?;

    // add 1 so we can account for the refund message that may be sent
    let mut bank_messages = Vec::with_capacity(messages.messages.len() + 1);
    for message in messages.messages {
        match &message.kind {
            DistributionKind::Burn(distribution_burn) => {
                dr_escrow.amount = dr_escrow.amount.checked_sub(distribution_burn.amount)?;
                bank_messages.push(BankMsg::Burn {
                    amount: vec![amount_to_tokens(distribution_burn.amount, token)],
                });
                event = event.add_attribute(
                    "burn",
                    to_json_string(&json!({
                        "amount": distribution_burn.amount,
                        "type": to_json_string(&message.type_)?,
                        "kind": to_json_string(&message.kind)?,
                    }))?,
                );
            }
            DistributionKind::ExecutorReward(distribution_executor_reward) => {
                let public_key = PublicKey::from_hex_str(&distribution_executor_reward.identity)?;
                let mut staker = STAKERS.get_staker(deps.storage, &public_key)?;

                let minimum_stake = STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                let remaining_reward = if staker.tokens_staked < minimum_stake {
                    // top the staker up to minimum stake from the amount in the reward & escrow
                    let top_up = minimum_stake.checked_sub(staker.tokens_staked)?;
                    let top_up = top_up.min(distribution_executor_reward.amount);
                    staker.tokens_staked += top_up;
                    dr_escrow.amount = dr_escrow.amount.checked_sub(top_up)?;
                    distribution_executor_reward.amount.checked_sub(top_up)?
                } else {
                    distribution_executor_reward.amount
                };

                // send remaining reward to the staker pending withdrawal
                staker.tokens_pending_withdrawal += remaining_reward;
                dr_escrow.amount = dr_escrow.amount.checked_sub(remaining_reward)?;
                STAKERS.update(deps.storage, public_key, &staker)?;

                event = event.add_attribute(
                    "executor_reward",
                    to_json_string(&json!({
                        "amount": distribution_executor_reward.amount,
                        "identity": distribution_executor_reward.identity,
                        "type": to_json_string(&message.type_)?,
                        "kind": to_json_string(&message.kind)?,
                    }))?,
                );
            }
            DistributionKind::Send(distribution_send) => {
                dr_escrow.amount = dr_escrow.amount.checked_sub(distribution_send.amount)?;
                bank_messages.push(BankMsg::Send {
                    to_address: Addr::from_vec(distribution_send.to.to_vec())?.to_string(),
                    amount:     vec![amount_to_tokens(distribution_send.amount, token)],
                });
                event = event.add_attribute(
                    "send",
                    to_json_string(&json!({
                        "amount": distribution_send.amount,
                        "to": distribution_send.to,
                        "type": to_json_string(&message.type_)?,
                        "kind": to_json_string(&message.kind)?,
                    }))?,
                );
            }
        }
    }

    if !dr_escrow.amount.is_zero() {
        bank_messages.push(BankMsg::Send {
            to_address: dr_escrow.poster.to_string(),
            amount:     vec![amount_to_tokens(dr_escrow.amount, token)],
        });
        event = event.add_attribute(
            "refund",
            to_json_string(&json!({
                "amount": dr_escrow.amount,
                "type": to_json_string(&messages.refund_type)?,
            }))?,
        );
        event = event.add_attribute("refund", dr_escrow.amount.to_string());
    }

    state::remove_request(deps.storage, dr_id)?;
    DR_ESCROW.remove(deps.storage, &dr_id);

    Ok((event, bank_messages))
}

impl SudoHandler for remove_requests::Sudo {
    fn sudo(self, mut deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let mut response = Response::new();
        for removal in self
            .requests
            .into_iter()
            .map(|(dr_id, messages)| remove_request(dr_id, messages, &mut deps, &env, &token))
        {
            let (event, bank_messages) = removal?;
            response = response.add_event(event).add_messages(bank_messages);
        }
        Ok(response)
    }
}
