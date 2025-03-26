use cosmwasm_std::Uint256;
use data_requests::state::load_request;

use super::{staking::state::STAKERS, *};

pub fn is_eligible_for_dr(deps: Deps, dr_id: [u8; 32], public_key: PublicKey) -> Result<bool, ContractError> {
    let data_request = load_request(deps.storage, &dr_id)?;
    let config = STAKING_CONFIG.load(deps.storage)?;

    let stakers = STAKERS.stakers.range_raw(deps.storage, None, None, Order::Ascending);
    let all_active_stakers = stakers
        .filter_map(|stakers_info| {
            if let Ok((public_key, staker)) = stakers_info {
                if staker.tokens_staked >= config.minimum_stake {
                    return Some((public_key, staker));
                }
            }

            None
        })
        .collect::<Vec<(Vec<u8>, Staker)>>();

    let (active_staker_index, _) = all_active_stakers
        .iter()
        .enumerate()
        .find(|(_, (pk, _staker))| public_key.as_ref() == pk.as_slice())
        .expect("Could not find staker");

    let executor_index = Uint256::from(active_staker_index as u64);
    let executor_length = Uint256::from(all_active_stakers.len() as u64);

    let dr_index = Uint256::from_be_bytes(dr_id) % executor_length;
    let replication_factor = Uint256::from(data_request.replication_factor);
    let end_index = (dr_index + replication_factor) % executor_length;

    if dr_index < end_index {
        // No overflow case
        Ok(executor_index >= dr_index && executor_index < end_index)
    } else {
        // Overflow case
        Ok(executor_index >= dr_index || executor_index < end_index)
    }
}
