use cosmwasm_std::Storage;
use cw_storage_plus::Bound;

use super::*;
mod data_requests_map;
use data_requests_map::{new_enumerable_status_map, DataRequestsMap};
mod timeouts;
use timeouts::Timeouts;

/// Governance-controlled timeout configuration parameters.
pub const TIMEOUT_CONFIG: Item<TimeoutConfig> = Item::new("timeout_config");

const DATA_REQUESTS: DataRequestsMap = new_enumerable_status_map!("data_request_pool");
const DATA_RESULTS: Map<&Hash, DataResult> = Map::new("data_results_pool");

pub fn init_data_requests(store: &mut dyn Storage) -> Result<(), ContractError> {
    Ok(DATA_REQUESTS.initialize(store)?)
}

pub fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.has(deps.storage, &dr_id) || DATA_RESULTS.has(deps.storage, &dr_id)
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

pub fn commit(store: &mut dyn Storage, block_height: u64, dr_id: Hash, dr: DataRequest) -> StdResult<()> {
    let status = if dr.reveal_started() {
        // We change the timeout to the reveal timeout or maybe this should move to the .update function?
        DATA_REQUESTS.timeouts.remove_by_dr_id(store, &dr_id)?;
        let timeout_config = TIMEOUT_CONFIG.load(store)?;
        DATA_REQUESTS
            .timeouts
            .insert(store, timeout_config.reveal_timeout_in_blocks + block_height, &dr_id)?;
        Some(DataRequestStatus::Revealing)
    } else {
        None
    };
    DATA_REQUESTS.update(store, dr_id, dr, status, false)?;

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

pub fn reveal(store: &mut dyn Storage, dr_id: Hash, dr: DataRequest) -> StdResult<()> {
    let status = if dr.is_tallying() {
        DATA_REQUESTS.timeouts.remove_by_dr_id(store, &dr_id)?;
        // We update the status of the request from Revealing to Tallying
        // So the chain can grab it and start tallying
        Some(DataRequestStatus::Tallying)
    } else {
        None
    };
    DATA_REQUESTS.update(store, dr_id, dr, status, false)?;

    Ok(())
}

pub fn post_result(store: &mut dyn Storage, dr_id: Hash, dr: &DataResult) -> StdResult<()> {
    // we have to remove the request from the pool and save it to the results
    DATA_RESULTS.save(store, &dr_id, dr)?;
    DATA_REQUESTS.remove(store, dr_id)?;
    // no need to update status as we remove it from the requests pool

    Ok(())
}

pub fn load_result(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataResult> {
    DATA_RESULTS.load(store, dr_id)
}

pub fn may_load_result(store: &dyn Storage, dr_id: &Hash) -> StdResult<Option<DataResult>> {
    DATA_RESULTS.may_load(store, dr_id)
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
