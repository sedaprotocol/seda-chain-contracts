use common::{
    msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse},
    state::DataRequest,
    types::{Bytes, Hash},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

pub mod data_requests {
    use std::collections::HashMap;

    use common::{error::ContractError, msg::PostDataRequestArgs};
    use cosmwasm_std::{Binary, Event};

    use super::*;
    use crate::{
        contract::CONTRACT_VERSION,
        state::{DATA_REQUESTS_POOL, DATA_RESULTS},
        utils::{hash_data_request, hash_to_string},
    };

    /// Internal function to return whether a data request or result exists with the given id.
    fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
        DATA_REQUESTS_POOL
            .may_load(deps.storage, dr_id)
            .ok()
            .flatten()
            .is_some()
            || DATA_RESULTS.may_load(deps.storage, &dr_id).ok().flatten().is_some()
    }

    /// Posts a data request to the pool
    pub fn post_data_request(
        deps: DepsMut,
        _info: MessageInfo,
        posted_dr: PostDataRequestArgs,
        seda_payload: Bytes,
        payback_address: Bytes,
    ) -> Result<Response, ContractError> {
        // hash the inputs to get the data request id
        let dr_id = hash_data_request(&posted_dr);

        // require the data request id to be unique
        if data_request_or_result_exists(deps.as_ref(), dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // TODO: check that the payback address is valid

        // TODO: review this event
        let res = Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(Binary::from(dr_id.to_vec()))
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(&dr_id)),
                ("dr_binary_id", &hash_to_string(&posted_dr.dr_binary_id)),
                ("tally_binary_id", &hash_to_string(&posted_dr.tally_binary_id)),
                ("dr_inputs", &serde_json::to_string(&posted_dr.dr_inputs).unwrap()),
                ("tally_inputs", &serde_json::to_string(&posted_dr.tally_inputs).unwrap()),
                ("memo", &serde_json::to_string(&posted_dr.memo).unwrap()),
                ("replication_factor", &posted_dr.replication_factor.to_string()),
                ("gas_price", &posted_dr.gas_price.to_string()),
                ("gas_limit", &posted_dr.gas_limit.to_string()),
                ("seda_payload", &serde_json::to_string(&seda_payload).unwrap()),
                ("payback_address", &serde_json::to_string(&payback_address).unwrap()),
            ]));

        // save the data request
        let dr = DataRequest {
            id: dr_id,
            version: posted_dr.version,
            dr_binary_id: posted_dr.dr_binary_id,
            dr_inputs: posted_dr.dr_inputs,
            tally_binary_id: posted_dr.tally_binary_id,
            tally_inputs: posted_dr.tally_inputs,
            replication_factor: posted_dr.replication_factor,
            gas_price: posted_dr.gas_price,
            gas_limit: posted_dr.gas_limit,
            memo: posted_dr.memo,

            payback_address,
            seda_payload,
            commits: HashMap::new(),
            reveals: HashMap::new(),
        };
        DATA_REQUESTS_POOL.add(deps.storage, dr_id, dr)?;

        Ok(res)
    }

    /// Returns a data request from the pool with the given id, if it exists.
    pub fn get_data_request(deps: Deps, dr_id: Hash) -> StdResult<GetDataRequestResponse> {
        let dr = DATA_REQUESTS_POOL.may_load(deps.storage, dr_id)?;
        Ok(GetDataRequestResponse { value: dr })
    }

    /// Returns a list of data requests from the pool, starting from the given position and limited by the given limit.
    pub fn get_data_requests_from_pool(
        deps: Deps,
        position: Option<u128>,
        limit: Option<u128>,
    ) -> StdResult<GetDataRequestsFromPoolResponse> {
        let position = position.unwrap_or(0);
        let dr_count = DATA_REQUESTS_POOL.len(deps.storage)?;
        let limit = limit.unwrap_or(dr_count);

        if position > dr_count {
            return Ok(GetDataRequestsFromPoolResponse { value: vec![] });
        }

        // compute the actual limit, taking into account the array size
        let actual_limit = (position + limit).clamp(position, dr_count);

        let mut requests = vec![];
        for i in position..actual_limit {
            let dr_id = DATA_REQUESTS_POOL.load_at_index(deps.storage, i)?;
            requests.push(DATA_REQUESTS_POOL.load(deps.storage, dr_id)?);
        }

        Ok(GetDataRequestsFromPoolResponse { value: requests })
    }
}
