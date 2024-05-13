#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};
use common::state::DataRequest;
use common::types::{Bytes, Hash};

pub mod data_requests {
    use crate::{contract::CONTRACT_VERSION, state::DATA_REQUESTS_POOL, utils::hash_to_string};
    use common::{error::ContractError, msg::PostDataRequestArgs};
    use cosmwasm_std::{Binary, Event};
    use std::collections::HashMap;

    use crate::{state::DATA_RESULTS, utils::hash_data_request};

    use super::*;

    /// Internal function to return whether a data request or result exists with the given id.
    fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
        DATA_REQUESTS_POOL
            .may_load(deps.storage, dr_id)
            .ok()
            .flatten()
            .is_some()
            || DATA_RESULTS
                .may_load(deps.storage, dr_id)
                .ok()
                .flatten()
                .is_some()
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

        // save the data request
        let dr = DataRequest {
            id: dr_id,
            version: posted_dr.clone().version,
            dr_binary_id: posted_dr.clone().dr_binary_id,
            dr_inputs: posted_dr.clone().dr_inputs,
            tally_binary_id: posted_dr.clone().tally_binary_id,
            tally_inputs: posted_dr.clone().tally_inputs,
            replication_factor: posted_dr.clone().replication_factor,
            gas_price: posted_dr.clone().gas_price,
            gas_limit: posted_dr.clone().gas_limit,
            memo: posted_dr.clone().memo,

            payback_address: payback_address.clone(),
            seda_payload: seda_payload.clone(),
            commits: HashMap::new(),
            reveals: HashMap::new(),
        };
        DATA_REQUESTS_POOL.add(deps.storage, dr_id, dr)?;

        // TODO: review this event
        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(Binary::from(dr_id.to_vec()))
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(dr_id)),
                (
                    "dr_binary_id",
                    &hash_to_string(posted_dr.clone().dr_binary_id),
                ),
                (
                    "tally_binary_id",
                    &hash_to_string(posted_dr.clone().tally_binary_id),
                ),
                (
                    "dr_inputs",
                    &serde_json::to_string(&posted_dr.clone().dr_inputs).unwrap(),
                ),
                (
                    "tally_inputs",
                    &serde_json::to_string(&posted_dr.clone().tally_inputs).unwrap(),
                ),
                (
                    "memo",
                    &serde_json::to_string(&posted_dr.clone().memo).unwrap(),
                ),
                (
                    "replication_factor",
                    &posted_dr.clone().replication_factor.to_string(),
                ),
                ("gas_price", &posted_dr.clone().gas_price.to_string()),
                ("gas_limit", &posted_dr.clone().gas_limit.to_string()),
                (
                    "seda_payload",
                    &serde_json::to_string(&seda_payload).unwrap(),
                ),
                (
                    "payback_address",
                    &serde_json::to_string(&payback_address).unwrap(),
                ),
            ])))
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

#[cfg(test)]
mod dr_tests {

    use super::*;
    use crate::contract::execute;
    use crate::helpers::calculate_dr_id_and_args;
    use crate::helpers::construct_dr;
    use crate::helpers::get_dr;
    use crate::helpers::get_drs_from_pool;
    use crate::helpers::instantiate_dr_contract;
    use common::error::ContractError;
    use common::msg::DataRequestsExecuteMsg as ExecuteMsg;
    use common::msg::GetDataRequestResponse;
    use common::types::SimpleHash;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    #[test]
    fn post_data_request() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(2, "token"));

        instantiate_dr_contract(deps.as_mut(), info.clone()).unwrap();

        // data request with id 0x69... does not yet exist
        let value: GetDataRequestResponse = get_dr(
            deps.as_mut(),
            "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".simple_hash(),
        );
        assert_eq!(None, value.value);

        let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

        let info = mock_info("anyone", &coins(2, "token"));

        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args,
            seda_payload: vec![],
            payback_address: vec![],
        };
        // someone posts a data request
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // expect an error when trying to post it again
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists));

        // should be able to fetch data request with id 0x69...
        let received_value: GetDataRequestResponse = get_dr(deps.as_mut(), constructed_dr_id);

        let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

        assert_eq!(
            Some(construct_dr(constructed_dr_id, dr_args, vec![])),
            received_value.value
        );

        // nonexistent data request does not yet exist

        let value: GetDataRequestResponse = get_dr(deps.as_mut(), "nonexistent".simple_hash());

        assert_eq!(None, value.value);
    }

    #[test]
    fn get_data_requests() {
        let mut deps = mock_dependencies();
        let info: MessageInfo = mock_info("creator", &coins(2, "token"));

        instantiate_dr_contract(deps.as_mut(), info).unwrap();

        let (_, dr_args1) = calculate_dr_id_and_args(1, 3);

        let (_, dr_args2) = calculate_dr_id_and_args(2, 3);

        let (_, dr_args3) = calculate_dr_id_and_args(3, 3);

        // someone posts three data requests
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args1,
            seda_payload: vec![],
            payback_address: vec![],
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args2,
            seda_payload: vec![],
            payback_address: vec![],
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args3,
            seda_payload: vec![],
            payback_address: vec![],
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let (constructed_dr_id1, dr_args1) = calculate_dr_id_and_args(1, 3);

        let (constructed_dr_id2, dr_args2) = calculate_dr_id_and_args(2, 3);

        let (constructed_dr_id3, dr_args3) = calculate_dr_id_and_args(3, 3);

        let constructed_dr1 = construct_dr(constructed_dr_id1, dr_args1, vec![]);
        let constructed_dr2 = construct_dr(constructed_dr_id2, dr_args2, vec![]);
        let constructed_dr3 = construct_dr(constructed_dr_id3, dr_args3, vec![]);

        // fetch all three data requests

        let response: GetDataRequestsFromPoolResponse =
            get_drs_from_pool(deps.as_mut(), None, None);

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    constructed_dr1.clone(),
                    constructed_dr2.clone(),
                    constructed_dr3.clone(),
                ]
            },
            response
        );

        // fetch data requests with limit of 2

        let response: GetDataRequestsFromPoolResponse =
            get_drs_from_pool(deps.as_mut(), None, Some(2));

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![constructed_dr1.clone(), constructed_dr2.clone(),]
            },
            response
        );

        // fetch a single data request

        let response: GetDataRequestsFromPoolResponse =
            get_drs_from_pool(deps.as_mut(), Some(1), Some(1));

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![constructed_dr2.clone()]
            },
            response
        );

        // fetch all data requests starting from id 1

        let response: GetDataRequestsFromPoolResponse =
            get_drs_from_pool(deps.as_mut(), Some(1), None);

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![constructed_dr2.clone(), constructed_dr3.clone(),]
            },
            response
        );
    }

    #[test]
    fn test_hash_data_request() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(2, "token"));

        // instantiate contract
        instantiate_dr_contract(deps.as_mut(), info).unwrap();

        let (constructed_dr_id, _) = calculate_dr_id_and_args(1, 3);

        println!("0x{}", hex::encode(constructed_dr_id));
    }
}
