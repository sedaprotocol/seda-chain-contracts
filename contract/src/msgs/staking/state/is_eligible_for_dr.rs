use cosmwasm_std::Uint256;
use data_requests::state::load_request;

use super::{staking::state::STAKERS, *};

pub fn is_eligible_for_dr(deps: Deps, dr_id: [u8; 32], public_key: PublicKey) -> Result<bool, ContractError> {
    let data_request = load_request(deps.storage, &dr_id)?;

    let executor_index = Uint256::from(STAKERS.public_keys.get_index(deps.storage, public_key)?);
    let executor_length = Uint256::from(STAKERS.len(deps.storage)?);
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
