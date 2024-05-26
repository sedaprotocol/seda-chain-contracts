use std::collections::HashMap;

use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Event, MessageInfo, Response};
use cw_storage_plus::KeyDeserialize;

use crate::{
    contract::CONTRACT_VERSION,
    error::ContractError,
    msgs::data_requests::{DataRequest, PostDataRequestArgs},
    state::DATA_REQUESTS,
    types::{Bytes, Hash, Hasher},
};

/// Internal function to return whether a data request or result exists with the given id.
fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
    DATA_REQUESTS.may_load(deps.storage, &dr_id).ok().flatten().is_some()
    // || DATA_RESULTS.may_load(deps.storage, &dr_id).ok().flatten().is_some()
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
    let dr_id = posted_dr.hash();

    // require the data request id to be unique
    if data_request_or_result_exists(deps.as_ref(), dr_id) {
        return Err(ContractError::DataRequestAlreadyExists);
    }

    let Ok(addr) = Addr::from_slice(&payback_address) else {
        return Err(ContractError::InvalidPaybackAddr);
    };

    // TODO: review this event
    let res = Response::new()
        .add_attribute("action", "post_data_request")
        .set_data(Binary::from(dr_id))
        .add_event(Event::new("seda-data-request").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("dr_id", dr_id.hash_hex()),
            ("dr_binary_id", posted_dr.dr_binary_id.hash_hex()),
            ("tally_binary_id", posted_dr.tally_binary_id.hash_hex()),
            // ("dr_inputs", &serde_json::to_string(&posted_dr.dr_inputs).unwrap()),
            // ("tally_inputs", &serde_json::to_string(&posted_dr.tally_inputs).unwrap()),
            // ("memo", &serde_json::to_string(&posted_dr.memo).unwrap()),
            ("replication_factor", posted_dr.replication_factor.to_string()),
            ("gas_price", posted_dr.gas_price.to_string()),
            ("gas_limit", posted_dr.gas_limit.to_string()),
            // ("seda_payload", &serde_json::to_string(&seda_payload).unwrap()),
            ("payback_address", addr.into_string()),
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
    DATA_REQUESTS.save(deps.storage, &dr_id, &dr)?;

    Ok(res)
}
