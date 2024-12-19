use cosmwasm_schema::cw_serde;
use cosmwasm_std::Storage;
use cw_storage_plus::Bound;

use super::*;
mod data_requests_map;
use data_requests_map::{new_enumerable_status_map, DataRequestsMap};
mod timeouts;
use timeouts::Timeouts;

/// Governance-controlled timeout configuration parameters.
pub const TIMEOUT_CONFIG: Item<TimeoutConfig> = Item::new("timeout_config");

/// Stores the amount, and the staker address.
#[cw_serde]
pub struct Escrow {
    pub amount: Uint128,
    // Safe to use Addr here as we aren't taking the the type from a user input.
    pub staker: Addr,
}

/// Maps a data request ID to the staked funds.
pub const DR_ESCROW: Map<&Hash, Escrow> = Map::new("dr_staked_funds");

const DATA_REQUESTS: DataRequestsMap = new_enumerable_status_map!("data_request_pool");

pub fn init_data_requests(store: &mut dyn Storage) -> Result<(), ContractError> {
    Ok(DATA_REQUESTS.initialize(store)?)
}

pub fn data_request_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.has(deps.storage, &dr_id)
}

pub fn may_load_request(store: &dyn Storage, dr_id: &Hash) -> StdResult<Option<DataRequest>> {
    DATA_REQUESTS.may_get(store, dr_id)
}

pub fn load_request(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataRequest> {
    DATA_REQUESTS.get(store, dr_id)
}

pub fn get_dr_expiration_height(store: &dyn Storage, dr_id: &Hash) -> StdResult<u64> {
    DATA_REQUESTS.timeouts.get_timeout_by_dr_id(store, dr_id)
}

pub fn post_request(
    store: &mut dyn Storage,
    current_height: u64,
    dr_id: Hash,
    dr: DataRequest,
) -> Result<(), ContractError> {
    // insert the data request
    DATA_REQUESTS.insert(store, current_height, dr_id, dr, &DataRequestStatus::Committing)?;

    Ok(())
}

pub fn commit(store: &mut dyn Storage, current_height: u64, dr_id: Hash, dr: DataRequest) -> StdResult<()> {
    let status = if dr.reveal_started() {
        Some(DataRequestStatus::Revealing)
    } else {
        None
    };
    DATA_REQUESTS.update(store, dr_id, dr, status, current_height, false)?;

    Ok(())
}

pub fn requests_by_status(
    store: &dyn Storage,
    status: &DataRequestStatus,
    offset: u32,
    limit: u32,
) -> StdResult<Vec<DataRequest>> {
    DATA_REQUESTS.get_requests_by_status(store, status, offset, limit)
}

pub fn reveal(store: &mut dyn Storage, dr_id: Hash, dr: DataRequest, current_height: u64) -> StdResult<()> {
    let status = if dr.is_tallying() {
        // We update the status of the request from Revealing to Tallying
        // So the chain can grab it and start tallying
        Some(DataRequestStatus::Tallying)
    } else {
        None
    };
    DATA_REQUESTS.update(store, dr_id, dr, status, current_height, false)?;

    Ok(())
}

pub fn remove_request(store: &mut dyn Storage, dr_id: Hash) -> StdResult<()> {
    // we have to remove the request from the pool
    DATA_REQUESTS.remove(store, dr_id)?;
    // no need to update status as we remove it from the requests pool

    Ok(())
}

pub fn expire_data_requests(store: &mut dyn Storage, current_height: u64) -> StdResult<Vec<String>> {
    DATA_REQUESTS.expire_data_requests(store, current_height)
}

#[cfg(test)]
#[path = ""]
mod tests {
    use super::*;
    mod data_requests_map_tests;
    mod timeouts_tests;
}
