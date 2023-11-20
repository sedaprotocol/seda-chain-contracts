#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};
use common::state::DataRequest;
use common::types::Hash;

pub mod data_requests {
    use crate::{contract::CONTRACT_VERSION, state::DATA_REQUESTS_POOL, utils::{hash_seed, hash_to_string}, data_request_result::data_request_results::get_seed};
    use common::{consts::ZERO_HASH, error::ContractError, msg::PostDataRequestArgs};
    use cosmwasm_std::{Binary, Event};
    use std::collections::HashMap;

    use crate::{
        state::{DataRequestInputs, DATA_RESULTS},
        utils::hash_data_request,
    };

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
    ) -> Result<Response, ContractError> {
        // require the data request id to be unique
        if data_request_or_result_exists(deps.as_ref(), posted_dr.dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // require dr_binary_id and tally_binary_id to be non-empty
        if posted_dr.dr_binary_id == ZERO_HASH {
            return Err(ContractError::EmptyArg("dr_binary_id".to_string()));
        }
        if posted_dr.tally_binary_id == ZERO_HASH {
            return Err(ContractError::EmptyArg("tally_binary_id".to_string()));
        }

        let dr_inputs = DataRequestInputs {
            dr_binary_id: posted_dr.dr_binary_id,
            tally_binary_id: posted_dr.tally_binary_id,
            dr_inputs: posted_dr.dr_inputs.clone(),
            tally_inputs: posted_dr.tally_inputs.clone(),
            memo: posted_dr.memo.clone(),
            replication_factor: posted_dr.replication_factor,

            gas_price: posted_dr.gas_price,
            gas_limit: posted_dr.gas_limit,

            seda_payload: posted_dr.seda_payload.clone(),
            payback_address: posted_dr.payback_address.clone(),
        };

        let reconstructed_dr_id = hash_data_request(dr_inputs);

        // check if the reconstructed dr_id matches the given dr_id
        if reconstructed_dr_id != posted_dr.dr_id {
            return Err(ContractError::InvalidDataRequestId(
                reconstructed_dr_id,
                posted_dr.dr_id,
            ));
        }
        let dr_id = posted_dr.dr_id.clone();
        let block_seed = hash_seed(get_seed(deps.as_ref())?.seed, dr_id.clone());
        // save the data request
        let dr = DataRequest {
            dr_id,
            dr_binary_id: posted_dr.dr_binary_id.clone(),
            tally_binary_id: posted_dr.tally_binary_id.clone(),
            dr_inputs: posted_dr.dr_inputs.clone(),
            tally_inputs: posted_dr.tally_inputs.clone(),
            memo: posted_dr.memo.clone(),
            replication_factor: posted_dr.replication_factor,
            gas_price: posted_dr.gas_price,
            gas_limit: posted_dr.gas_limit,
            seda_payload: posted_dr.seda_payload.clone(),
            payback_address: posted_dr.payback_address.clone(),
            commits: HashMap::new(),
            reveals: HashMap::new(),
            block_seed,
        };
        DATA_REQUESTS_POOL.add(deps.storage, posted_dr.dr_id, dr)?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(Binary::from(posted_dr.dr_id.to_vec()))
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(posted_dr.dr_id)),
                ("dr_binary_id", &hash_to_string(posted_dr.dr_binary_id)),
                (
                    "tally_binary_id",
                    &hash_to_string(posted_dr.tally_binary_id),
                ),
                (
                    "dr_inputs",
                    &serde_json::to_string(&posted_dr.dr_inputs).unwrap(),
                ),
                (
                    "tally_inputs",
                    &serde_json::to_string(&posted_dr.tally_inputs).unwrap(),
                ),
                ("memo", &serde_json::to_string(&posted_dr.memo).unwrap()),
                (
                    "replication_factor",
                    &posted_dr.replication_factor.to_string(),
                ),
                ("gas_price", &posted_dr.gas_price.to_string()),
                ("gas_limit", &posted_dr.gas_limit.to_string()),
                (
                    "seda_payload",
                    &serde_json::to_string(&posted_dr.seda_payload).unwrap(),
                ),
                (
                    "payback_address",
                    &serde_json::to_string(&posted_dr.payback_address).unwrap(),
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
    use crate::utils::string_to_hash;
    use common::consts::ZERO_HASH;
    use common::error::ContractError;
    use common::msg::DataRequestsExecuteMsg as ExecuteMsg;
    use common::msg::GetDataRequestResponse;
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
            string_to_hash("0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f"),
        );
        assert_eq!(None, value.value);

        let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

        let info = mock_info("anyone", &coins(2, "token"));

        let msg = ExecuteMsg::PostDataRequest { posted_dr: dr_args };
        // someone posts a data request
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // expect an error when trying to post it again
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(
            res.is_err_and(|x| x == ContractError::DataRequestAlreadyExists),
            true
        );

        // should be able to fetch data request with id 0x69...
        let received_value: GetDataRequestResponse =
            get_dr(deps.as_mut(), constructed_dr_id.clone());

        let (constructed_dr_id, dr_args) = calculate_dr_id_and_args(1, 3);

        assert_eq!(
            Some(construct_dr(deps.as_ref(), constructed_dr_id, dr_args)),
            received_value.value
        );

        // nonexistent data request does not yet exist

        let value: GetDataRequestResponse = get_dr(deps.as_mut(), string_to_hash("nonexistent"));

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
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args2,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::PostDataRequest {
            posted_dr: dr_args3,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let (constructed_dr_id1, dr_args1) = calculate_dr_id_and_args(1, 3);

        let (constructed_dr_id2, dr_args2) = calculate_dr_id_and_args(2, 3);

        let (constructed_dr_id3, dr_args3) = calculate_dr_id_and_args(3, 3);

        let constructed_dr1 = construct_dr(deps.as_ref(), constructed_dr_id1, dr_args1);
        let constructed_dr2 = construct_dr(deps.as_ref(), constructed_dr_id2, dr_args2);
        let constructed_dr3 = construct_dr(deps.as_ref(), constructed_dr_id3, dr_args3);

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

        println!("constructed_dr_id1: {:#?}", constructed_dr_id);
    }

    #[test]
    #[should_panic(expected = "InvalidDataRequestId")]
    fn invalid_data_request_id() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(2, "token"));

        // instantiate contract
        instantiate_dr_contract(deps.as_mut(), info).unwrap();

        // calculate args then modify the dr_id to be incorrect
        let (_, mut posted_dr) = calculate_dr_id_and_args(1, 3);
        posted_dr.dr_id = string_to_hash("invalid hash");

        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let info = mock_info("anyone", &coins(2, "token"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    #[should_panic(expected = "EmptyArg")]
    fn empty_arg() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(2, "token"));

        // instantiate contract
        instantiate_dr_contract(deps.as_mut(), info).unwrap();

        // calculate args then modify the dr_binary_id to be empty
        let (_, mut posted_dr) = calculate_dr_id_and_args(1, 3);
        posted_dr.dr_binary_id = ZERO_HASH;

        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let info = mock_info("anyone", &coins(2, "token"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
