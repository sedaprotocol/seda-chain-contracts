use std::collections::HashSet;

use cosmwasm_std::{to_json_string, BankMsg, Coin, DepsMut, Env, Event, Response, Uint128};
use seda_common::{
    msgs::data_requests::sudo::{remove_requests, DistributionMessage},
    types::{Hash, ToHexStr},
};
use serde_json::json;

use super::{ContractError, SudoHandler};
use crate::{
    msgs::{
        data_requests::state::{self, Escrow, DR_ESCROW},
        staking::{
            execute::staking_events::create_executor_event,
            state::{STAKERS, STAKING_CONFIG},
        },
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

fn burn(amount: Uint128, token: &str, escrow: &mut Escrow) -> BankMsg {
    escrow.amount = escrow.amount.saturating_sub(amount);

    BankMsg::Burn {
        amount: vec![amount_to_tokens(amount, token)],
    }
}

fn remove_request_and_process_distributions(
    dr_id_str: String,
    messages: &[DistributionMessage],
    deps: &mut DepsMut,
    token: &str,
) -> Result<(Event, Vec<BankMsg>, HashSet<PublicKey>), ContractError> {
    // find the data request from the committed pool (if it exists, otherwise error)
    let dr_id = Hash::from_hex_str(&dr_id_str)?;
    let dr = state::load_request(deps.storage, &dr_id)?;

    let mut event = Event::new("seda-remove-dr")
        .add_attributes([("dr_id", dr_id_str), ("posted_dr_height", dr.height.to_string())]);

    let mut dr_escrow = DR_ESCROW.load(deps.storage, &dr_id)?;

    // add 1 so we can account for the refund message that may be sent
    let mut bank_messages = Vec::new();
    let mut stakers_effected = HashSet::new();

    // We need to send messages in the order given.
    'process_message: for message in messages {
        // No reason to keep processing if the escrowed amount is zero
        if dr_escrow.amount.is_zero() {
            event = event.add_attribute("escrow-emptied-early", "true");
            break;
        }

        // Regardless of the message type we first need to get the min of the escrowed amount and the message amount
        // as this will prevent overflows and over-sending of tokens.
        match &message {
            DistributionMessage::Burn(distribution_burn) => {
                let amount_to_burn = distribution_burn.amount.min(dr_escrow.amount);
                bank_messages.push(burn(amount_to_burn, token, &mut dr_escrow));
                event = event.add_attribute(
                    "burn",
                    to_json_string(&json!({
                        "amount": amount_to_burn,
                    }))?,
                );
            }
            DistributionMessage::DataProxyReward(distribution_send) => {
                let amount_to_reward = distribution_send.amount.min(dr_escrow.amount);

                if let Ok(addr) = deps.api.addr_validate(&distribution_send.payout_address) {
                    bank_messages.push(BankMsg::Send {
                        to_address: addr.to_string(),
                        amount:     vec![amount_to_tokens(amount_to_reward, token)],
                    });
                    dr_escrow.amount = dr_escrow.amount.saturating_sub(amount_to_reward);

                    event = event.add_attribute(
                        "data_proxy_reward",
                        to_json_string(&json!({
                            "amount": amount_to_reward,
                            "payout_address": distribution_send.payout_address,
                        }))?,
                    );
                } else {
                    bank_messages.push(burn(amount_to_reward, token, &mut dr_escrow));
                    event = event.add_attribute(
                        "data_proxy_reward_invalid_address",
                        to_json_string(&json!({
                            "payout_address": distribution_send.payout_address,
                            "burn_amount": amount_to_reward,
                        }))?,
                    );
                }
            }
            DistributionMessage::ExecutorReward(distribution_executor_reward) => {
                let amount_to_reward = distribution_executor_reward.amount.min(dr_escrow.amount);

                let Ok(public_key) = PublicKey::from_hex_str(&distribution_executor_reward.identity) else {
                    bank_messages.push(burn(amount_to_reward, token, &mut dr_escrow));
                    event = event.add_attribute(
                        "executor_reward_invalid_identity",
                        to_json_string(&json!({
                            "identity": distribution_executor_reward.identity,
                            "burn_amount": amount_to_reward,
                        }))?,
                    );
                    continue 'process_message;
                };

                let Ok(mut staker) = STAKERS.get_staker(deps.storage, &public_key) else {
                    bank_messages.push(burn(amount_to_reward, token, &mut dr_escrow));
                    event = event.add_attribute(
                        "executor_reward_invalid_identity",
                        to_json_string(&json!({
                            "identity": distribution_executor_reward.identity,
                            "burn_amount": amount_to_reward,
                        }))?,
                    );
                    continue 'process_message;
                };

                let minimum_stake = STAKING_CONFIG.load(deps.storage)?.minimum_stake_to_register;
                let (remaining_reward, topped_up) = if staker.tokens_staked < minimum_stake {
                    // top the staker up to minimum stake from the amount in the reward & escrow
                    let top_up = minimum_stake.saturating_sub(staker.tokens_staked);
                    let top_up = top_up.min(amount_to_reward);
                    staker.tokens_staked += top_up;
                    dr_escrow.amount = dr_escrow.amount.saturating_sub(top_up);

                    // Remaining reward after top-up
                    (amount_to_reward.saturating_sub(top_up), top_up)
                } else {
                    (amount_to_reward, 0u128.into())
                };

                // send remaining reward to the staker pending withdrawal
                staker.tokens_pending_withdrawal += remaining_reward;
                dr_escrow.amount = dr_escrow.amount.saturating_sub(remaining_reward);
                STAKERS.update(deps.storage, public_key.clone(), &staker)?;
                stakers_effected.insert(public_key);

                event = event.add_attribute(
                    "executor_reward",
                    to_json_string(&json!({
                        "amount": remaining_reward,
                        "topped_up": topped_up,
                        "identity": distribution_executor_reward.identity,
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
        event = event.add_attribute("refund", dr_escrow.amount.to_string());
    }

    state::remove_request(deps.storage, dr_id)?;
    DR_ESCROW.remove(deps.storage, &dr_id);

    Ok((event, bank_messages, stakers_effected))
}

impl SudoHandler for remove_requests::Sudo {
    fn sudo(self, mut deps: DepsMut, _: Env) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let mut response = Response::new();

        let mut all_stakers_effected = HashSet::new();
        for removal in self
            .requests
            .into_iter()
            .map(|(dr_id, messages)| remove_request_and_process_distributions(dr_id, &messages, &mut deps, &token))
        {
            let (event, bank_messages, stakers_effected) = removal?;
            all_stakers_effected.extend(stakers_effected);
            response = response.add_event(event).add_messages(bank_messages);
        }

        for staker in all_stakers_effected {
            response = response.add_event(create_executor_event(
                STAKERS.get_staker(deps.storage, &staker)?,
                staker.to_hex(),
            ));
        }

        Ok(response)
    }
}
