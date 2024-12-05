use cosmwasm_std::Uint256;
use data_requests::state::may_load_request;

use super::{staking::state::STAKERS, *};

pub fn is_eligible_for_dr(deps: Deps, dr_id: [u8; 32], public_key: PublicKey) -> Result<bool, ContractError> {
    let data_request = may_load_request(deps.storage, &dr_id)?.unwrap();
    let stakers = STAKERS.stakers.keys_raw(deps.storage, None, None, Order::Ascending);
    let all_stakers = stakers.collect::<Vec<Vec<u8>>>();

    let stakers_length: u64 = all_stakers.len().try_into().expect("cannot convert to u64");
    let (staker_index, _) = all_stakers
        .iter()
        .enumerate()
        .find(|(_, pk)| public_key.as_ref() == pk.as_slice())
        .expect("Could not find staker");

    let executor_index = Uint256::from(u128::try_from(staker_index).unwrap());
    let executor_length = Uint256::from(stakers_length);
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
