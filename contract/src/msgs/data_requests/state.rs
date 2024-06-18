use types::DataRequestsMap;

use super::*;
use crate::enumerable_status_map;

const DATA_REQUESTS: DataRequestsMap = enumerable_status_map!("data_request_pool");
const DATA_RESULTS: Map<&Hash, DataResult> = Map::new("data_results_pool");

pub fn init_data_requests(store: &mut dyn Storage) -> Result<(), ContractError> {
    Ok(DATA_REQUESTS.initialize(store)?)
}

pub fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.has(deps.storage, &dr_id) || DATA_RESULTS.has(deps.storage, &dr_id)
}

pub fn may_get_request(store: &dyn Storage, dr_id: &Hash) -> StdResult<Option<DataRequest>> {
    DATA_REQUESTS.may_get_by_key(store, dr_id)
}

pub fn load_request(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataRequest> {
    DATA_REQUESTS.get_by_key(store, dr_id)
}

pub fn insert_request(store: &mut dyn Storage, dr_id: &Hash, dr: DataRequest) -> Result<(), ContractError> {
    // insert the data request
    DATA_REQUESTS.insert(store, dr_id, dr)?;

    Ok(())
}

pub fn commit(store: &mut dyn Storage, dr_id: &Hash, dr: DataRequest) -> StdResult<()> {
    let status = if dr.reveal_started() {
        Some(DataRequestStatus::Revealing)
    } else {
        None
    };
    DATA_REQUESTS.update(store, dr_id, dr, status)?;

    Ok(())
}

pub fn requests_by_status(
    store: &dyn Storage,
    status: DataRequestStatus,
    offset: u32,
    limit: u32,
) -> StdResult<Vec<DataRequest>> {
    DATA_REQUESTS.get_requests_by_status(store, status, offset, limit)
}

pub fn reveal(storage: &mut dyn Storage, dr_id: &Hash, dr: DataRequest) -> StdResult<()> {
    let status = if dr.is_tallying() {
        // We update the status of the request from Revealing to Tallying
        // So the chain can grab it and start tallying
        Some(DataRequestStatus::Tallying)
    } else {
        None
    };
    DATA_REQUESTS.update(storage, dr_id, dr, status)?;

    Ok(())
}

pub fn post_result(store: &mut dyn Storage, dr_id: &Hash, dr: &DataResult) -> StdResult<()> {
    // we have to remove the request from the pool and save it to the results
    DATA_REQUESTS.swap_remove(store, dr_id)?;
    DATA_RESULTS.save(store, dr_id, dr)?;
    // no need to update status as we remove it from the requests pool

    Ok(())
}

pub fn load_result(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataResult> {
    DATA_RESULTS.load(store, dr_id)
}

pub fn may_load_resuslt(store: &dyn Storage, dr_id: &Hash) -> StdResult<Option<DataResult>> {
    DATA_RESULTS.may_load(store, dr_id)
}
