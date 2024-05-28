use super::{state::DATA_REQUESTS, *};

/// Internal function to return whether a data request or result exists with the given id.
pub fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.may_load(deps.storage, &dr_id).ok().flatten().is_some()
    // || DATA_RESULTS.may_load(deps.storage, &dr_id).ok().flatten().is_some()
}

pub fn get_dr(deps: Deps, dr_id: &Hash) -> StdResult<Option<DataRequest>> {
    DATA_REQUESTS.may_load(deps.storage, dr_id)
}
