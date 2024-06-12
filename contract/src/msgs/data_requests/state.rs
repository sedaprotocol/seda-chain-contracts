use types::EnumerableStatusMap;

use super::*;
use crate::enumerable_status_map;

const DATA_REQUESTS: EnumerableStatusMap = enumerable_status_map!("data_request_pool");
const DATA_RESULTS: Map<&Hash, DataResult> = Map::new("data_results_pool");

pub fn init_data_requests(store: &mut dyn Storage) -> Result<(), ContractError> {
    Ok(DATA_REQUESTS.initialize(store)?)
}

pub fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.has(deps.storage, &dr_id) || DATA_RESULTS.has(deps.storage, &dr_id)
}

pub fn may_load_req(store: &dyn Storage, dr_id: &Hash) -> StdResult<Option<DataRequest>> {
    DATA_REQUESTS.may_get_by_key(store, dr_id)
}

pub fn load_req(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataRequest> {
    DATA_REQUESTS.get_by_key(store, dr_id)
}

fn update_req_status(
    store: &mut dyn Storage,
    dr_id: &Hash,
    current_status: &DataRequestStatus,
    new_status: &DataRequestStatus,
) -> StdResult<()> {
    // // Load current statuses set
    // let mut current = DATA_REQUESTS_BY_STATUS
    //     .may_load(store, current_status)?
    //     .unwrap_or_default();

    // // Check if the request is in the current status set
    // if current.swap_remove(dr_id) {
    //     // If it was, save the updated set back without the request
    //     DATA_REQUESTS_BY_STATUS.save(store, current_status, &current)?;

    //     // Load or initialize the new status set
    //     let mut new = DATA_REQUESTS_BY_STATUS.may_load(store, new_status)?.unwrap_or_default();

    //     // Add the request to the new status set
    //     new.insert(*dr_id);
    //     DATA_REQUESTS_BY_STATUS.save(store, new_status, &new)?;
    // }
    Ok(())
}

pub fn insert_req(store: &mut dyn Storage, dr_id: &Hash, dr: DataRequest) -> Result<(), ContractError> {
    // insert the data request
    DATA_REQUESTS.insert(store, dr_id, dr)?;

    // set the status to AwaitingCommits
    // let mut statuses = DATA_REQUESTS_BY_STATUS
    //     .may_load(store, &DataRequestStatus::Committing)?
    //     .unwrap_or_default();
    // statuses.insert(*dr_id);
    // DATA_REQUESTS_BY_STATUS.save(store, &DataRequestStatus::Committing, &statuses)?;

    Ok(())
}

pub fn commit(store: &mut dyn Storage, dr_id: &Hash, dr: &DataRequest) -> StdResult<()> {
    // DATA_REQUESTS.update(store, dr_id, dr)?;

    if dr.reveal_started() {
        update_req_status(
            store,
            dr_id,
            &DataRequestStatus::Committing,
            &DataRequestStatus::Revealing,
        )?;
    }

    Ok(())
}

pub fn requests_by_status(
    store: &dyn Storage,
    status: DataRequestStatus,
    offset: u32,
    limit: u32,
) -> StdResult<HashMap<String, DataRequest>> {
    DATA_REQUESTS.get_requests_by_status(store, status, offset, limit)
}

pub fn save(storage: &mut dyn Storage, dr_id: &Hash, dr: &DataRequest) -> StdResult<()> {
    // DATA_REQUESTS.update(storage, dr_id, dr)?;

    // if dr.reveal_over() {
    //     // We update the status of the request from Revealing to Tallying
    //     // So the chain can grab it and start tallying
    //     update_req_status(
    //         storage,
    //         dr_id,
    //         &DataRequestStatus::Revealing,
    //         &DataRequestStatus::Tallying,
    //     )?;
    // }

    Ok(())
}

pub fn post_result(store: &mut dyn Storage, dr_id: &Hash, dr: &DataResult) -> StdResult<()> {
    // we have to remove the request from the pool and save it to the results
    DATA_REQUESTS.swap_remove(store, dr_id)?;
    DATA_RESULTS.save(store, dr_id, dr)?;
    // no need to update status as we remove it from the requests pool

    Ok(())
}

pub(crate) fn load_resolved_req(store: &dyn Storage, dr_id: &Hash) -> StdResult<DataResult> {
    DATA_RESULTS.load(store, dr_id)
}
