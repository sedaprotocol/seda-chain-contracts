#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Deps, DepsMut, MessageInfo, Order, Response, StdResult};

use crate::state::{DATA_REQUESTS, DATA_REQUESTS_COUNT};

use crate::error::ContractError;
use crate::msg::PostDataRequestResponse;
use common::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};
use common::state::DataRequest;
use common::types::Hash;

pub mod data_requests {
    use crate::contract::CONTRACT_VERSION;
    use common::msg::PostDataRequestArgs;
    use cosmwasm_std::Event;
    use std::collections::HashMap;

    use crate::{
        state::{DataRequestInputs, DATA_REQUESTS_BY_NONCE, DATA_RESULTS},
        utils::hash_data_request,
    };
    use cw_storage_plus::Bound;

    use super::*;

    /// Internal function to return whether a data request or result exists with the given id.
    fn data_request_or_result_exists(deps: Deps, dr_id: Hash) -> bool {
        DATA_REQUESTS
            .may_load(deps.storage, dr_id.clone())
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
        if data_request_or_result_exists(deps.as_ref(), posted_dr.dr_id.clone()) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // require dr_binary_id and tally_binary_id to be non-empty
        if posted_dr.dr_binary_id == *"" {
            return Err(ContractError::EmptyArg("dr_binary_id".to_string()));
        }
        if posted_dr.tally_binary_id == *"" {
            return Err(ContractError::EmptyArg("tally_binary_id".to_string()));
        }

        let dr_inputs = DataRequestInputs {
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
        };

        let reconstructed_dr_id = hash_data_request(dr_inputs);

        // check if the reconstructed dr_id matches the given dr_id
        if reconstructed_dr_id != posted_dr.dr_id {
            return Err(ContractError::InvalidDataRequestId(
                reconstructed_dr_id,
                posted_dr.dr_id,
            ));
        }

        // save the data request
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?;
        let dr = DataRequest {
            dr_id: posted_dr.dr_id.clone(),

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
        };
        DATA_REQUESTS.save(deps.storage, dr.dr_id.clone(), &dr)?;
        DATA_REQUESTS_BY_NONCE.save(deps.storage, dr_count, &posted_dr.dr_id)?; // todo wrong nonce

        // increment the data request count
        DATA_REQUESTS_COUNT.update(deps.storage, |mut new_dr_id| -> Result<_, ContractError> {
            new_dr_id += 1;
            Ok(new_dr_id)
        })?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(to_binary(&PostDataRequestResponse {
                dr_id: posted_dr.dr_id.clone(),
            })?)
            .add_event(Event::new("seda-data-request").add_attributes(vec![
                ("version", CONTRACT_VERSION),
                ("dr_id", &posted_dr.dr_id),
                ("dr_binary_id", &posted_dr.dr_binary_id),
                ("tally_binary_id", &posted_dr.tally_binary_id),
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
        let dr = DATA_REQUESTS.may_load(deps.storage, dr_id)?;
        Ok(GetDataRequestResponse { value: dr })
    }

    /// Returns a list of data requests from the pool, starting from the given position and limited by the given limit.
    pub fn get_data_requests_from_pool(
        deps: Deps,
        position: Option<u128>,
        limit: Option<u32>,
    ) -> StdResult<GetDataRequestsFromPoolResponse> {
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?.to_be_bytes();
        let position = position.unwrap_or(0).to_be_bytes();
        let limit = limit.unwrap_or(u32::MAX);

        // starting from position, iterate forwards until we reach the limit or the end of the data requests
        let mut requests = vec![];
        for dr in DATA_REQUESTS_BY_NONCE.range(
            deps.storage,
            Some(Bound::InclusiveRaw(position.into())),
            Some(Bound::ExclusiveRaw(dr_count.into())),
            Order::Ascending,
        ) {
            let dr_pending = DATA_REQUESTS.may_load(deps.storage, dr?.1)?;
            // skip if the data request is no longer in the pool
            if dr_pending.is_none() {
                continue;
            }
            requests.push(dr_pending.unwrap());
            if requests.len() == limit as usize {
                break;
            }
        }

        Ok(GetDataRequestsFromPoolResponse { value: requests })
    }
}

#[cfg(test)]
mod dr_tests {
    use std::collections::HashMap;

    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::msg::InstantiateMsg;
    use crate::state::DataRequestInputs;
    use crate::utils::hash_data_request;
    use common::msg::DataRequestsExecuteMsg as ExecuteMsg;
    use common::msg::DataRequestsQueryMsg as QueryMsg;
    use common::msg::GetDataRequestResponse;
    use common::msg::PostDataRequestArgs;
    use common::state::Reveal;
    use common::types::Bytes;
    use common::types::Commitment;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    use sha3::Digest;
    use sha3::Keccak256;

    #[test]
    fn post_data_request() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request with id 0x69... does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);
        let dr_binary_id: Hash = "dr_binary_id".to_string();
        let tally_binary_id: Hash = "tally_binary_id".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();
        let payback_address: Bytes = Vec::new();

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 1;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo1 = hasher.finalize().to_vec();

        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };

        let constructed_dr_id = hash_data_request(dr_inputs1);
        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data request with id 0x69...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: constructed_dr_id.clone(),
            },
        )
        .unwrap();
        let received_value: GetDataRequestResponse = from_binary(&res).unwrap();

        let dr_binary_id: Hash = "dr_binary_id".to_string();
        let tally_binary_id: Hash = "tally_binary_id".to_string();
        let dr_inputs: Bytes = Vec::new();
        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();
        let commits: HashMap<String, Commitment> = HashMap::new();
        let reveals: HashMap<String, Reveal> = HashMap::new();

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 1;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo1 = hasher.finalize().to_vec();

        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };

        let constructed_dr_id = hash_data_request(dr_inputs1);
        let payback_address: Bytes = Vec::new();
        assert_eq!(
            Some(DataRequest {
                dr_id: constructed_dr_id.clone(),

                dr_binary_id: dr_binary_id.clone(),
                tally_binary_id,
                dr_inputs,
                tally_inputs,
                memo: memo1,
                replication_factor,
                gas_price,
                gas_limit,
                seda_payload,
                commits,
                reveals,
                payback_address,
            }),
            received_value.value
        );

        // nonexistent data request does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequest {
                dr_id: "nonexistent".to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);
    }

    #[test]
    fn get_data_requests() {
        let mut deps = mock_dependencies();

        // instantiate contract
        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let dr_binary_id: Hash = "dr_binary_id".to_string();
        let tally_binary_id: Hash = "tally_binary_id".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();
        let payback_address: Bytes = Vec::new();

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 1;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo1 = hasher.finalize().to_vec();

        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id1 = hash_data_request(dr_inputs1);

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 2;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo2 = hasher.finalize().to_vec();

        let dr_inputs2 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo2.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id2 = hash_data_request(dr_inputs2);

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 3;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo3 = hasher.finalize().to_vec();

        let dr_inputs3 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo3.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id3 = hash_data_request(dr_inputs3);

        let payback_address: Bytes = Vec::new();
        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id1.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        // someone posts three data requests
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id2.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),

            memo: memo2,
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id3.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),

            memo: memo3,
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetch all three data requests
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: None,
            },
        )
        .unwrap();

        let payback_address: Bytes = Vec::new();

        let dr_binary_id: Hash = "dr_binary_id".to_string();
        let tally_binary_id: Hash = "tally_binary_id".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();

        let commits: HashMap<String, Commitment> = HashMap::new();
        let reveals: HashMap<String, Reveal> = HashMap::new();

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 1;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo1 = hasher.finalize().to_vec();

        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo1.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id1 = hash_data_request(dr_inputs1);

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 2;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo2 = hasher.finalize().to_vec();

        let dr_inputs2 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo2.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id2 = hash_data_request(dr_inputs2);

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 3;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo3 = hasher.finalize().to_vec();

        let dr_inputs3 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo3.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id3 = hash_data_request(dr_inputs3);

        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        dr_id: constructed_dr_id1.clone(),

                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo1.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone(),
                    },
                    DataRequest {
                        dr_id: constructed_dr_id2.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo2.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone(),
                    },
                    DataRequest {
                        dr_id: constructed_dr_id3.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo3.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone(),
                    },
                ]
            },
            response
        );

        // fetch data requests with limit of 2
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        let payback_address: Bytes = Vec::new();

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        dr_id: constructed_dr_id1.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo1.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone()
                    },
                    DataRequest {
                        dr_id: constructed_dr_id2.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo2.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone()
                    },
                ]
            },
            response
        );

        // fetch a single data request
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: Some(1),
                limit: Some(1),
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        let payback_address: Bytes = Vec::new();

        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![DataRequest {
                    dr_id: constructed_dr_id2.clone(),
                    dr_binary_id: dr_binary_id.clone(),
                    tally_binary_id: tally_binary_id.clone(),
                    dr_inputs: dr_inputs.clone(),
                    tally_inputs: tally_inputs.clone(),

                    memo: memo2.clone(),
                    replication_factor,
                    gas_price,
                    gas_limit,
                    seda_payload: seda_payload.clone(),
                    commits: commits.clone(),
                    reveals: reveals.clone(),
                    payback_address: payback_address.clone(),
                },]
            },
            response
        );

        // fetch all data requests starting from id 1
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: Some(1),
                limit: None,
            },
        )
        .unwrap();
        let response: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
        assert_eq!(
            GetDataRequestsFromPoolResponse {
                value: vec![
                    DataRequest {
                        dr_id: constructed_dr_id2.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo2.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone()
                    },
                    DataRequest {
                        dr_id: constructed_dr_id3.clone(),
                        dr_binary_id: dr_binary_id.clone(),
                        tally_binary_id: tally_binary_id.clone(),
                        dr_inputs: dr_inputs.clone(),
                        tally_inputs: tally_inputs.clone(),

                        memo: memo3.clone(),
                        replication_factor,
                        gas_price,
                        gas_limit,
                        seda_payload: seda_payload.clone(),
                        commits: commits.clone(),
                        reveals: reveals.clone(),
                        payback_address: payback_address.clone()
                    },
                ]
            },
            response
        );
    }

    #[test]
    fn test_hash_data_request() {
        let mut deps = mock_dependencies();

        // instantiate contract
        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let dr_binary_id: Hash = "dr_binary_id".to_string();
        let dr_inputs: Bytes = "dr_inputs".to_string().into_bytes();
        let tally_binary_id: Hash = "tally_binary_id".to_string();
        let tally_inputs: Bytes = "tally_inputs".to_string().into_bytes();

        let replication_factor: u16 = 123;

        // set by dr creator
        let gas_price: u128 = 456;
        let gas_limit: u128 = 789;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = "seda_payload".to_string().into_bytes();
        let payback_address: Bytes = "payback_address".to_string().into_bytes();

        // memo
        let chain_id: u128 = 31337;
        let nonce: u128 = 1;
        let mut hasher = Keccak256::new();
        hasher.update(chain_id.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        let memo = hasher.finalize().to_vec();

        // format inputs
        let dr_inputs1 = DataRequestInputs {
            dr_binary_id,
            tally_binary_id,
            dr_inputs,
            tally_inputs,
            memo,
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload,
            payback_address,
        };

        // reconstruct dr_id
        let constructed_dr_id1 = hash_data_request(dr_inputs1);
        println!("constructed_dr_id1: {}", constructed_dr_id1);
    }
}
