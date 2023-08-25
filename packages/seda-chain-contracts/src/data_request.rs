#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Order, Response, StdResult};
use cw_storage_plus::Bound;

use crate::state::{DATA_REQUESTS, DATA_REQUESTS_COUNT};

use crate::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};
use crate::state::DataRequest;
use crate::state::{DATA_REQUESTS_BY_NONCE, DATA_REQUESTS_COUNT, DATA_REQUESTS_POOL, DATA_RESULTS};
use crate::types::Hash;
use crate::utils::hash_data_request;
use crate::ContractError;

pub mod data_requests {
    use std::collections::HashMap;

    use cw_storage_plus::Bound;

    use crate::{
        msg::PostDataRequestArgs,
        state::{DATA_REQUESTS_BY_NONCE, DATA_RESULTS},
    };

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
<<<<<<< HEAD
        if data_request_or_result_exists(deps.as_ref(), args.dr_id.clone()) {
=======
        if data_request_or_result_exists(deps.as_ref(), posted_dr.dr_id.clone()) {
>>>>>>> refactor: add PostDataRequestArgs
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // reconstruct the data request id hash
        let mut hasher = Keccak256::new();
        hasher.update(posted_dr.dr_binary_id.clone());
        hasher.update(posted_dr.dr_inputs.clone());
        hasher.update(posted_dr.gas_limit.to_be_bytes());
        hasher.update(posted_dr.gas_price.to_be_bytes());
        hasher.update(posted_dr.memo.clone());
        hasher.update(posted_dr.payback_address.clone());
        hasher.update(posted_dr.replication_factor.to_be_bytes());
        hasher.update(posted_dr.seda_payload.clone());
        hasher.update(posted_dr.tally_binary_id.clone());
        hasher.update(posted_dr.tally_inputs.clone());

        let reconstructed_dr_id = format!("0x{}", hex::encode(hasher.finalize()));

        // check if the reconstructed dr_id matches the given dr_id
<<<<<<< HEAD
        if reconstructed_dr_id != args.dr_id {
            return Err(ContractError::InvalidDataRequestId(
                reconstructed_dr_id,
                args.dr_id,
=======
        if reconstructed_dr_id != posted_dr.dr_id {
            return Err(ContractError::InvalidDataRequestId(
                reconstructed_dr_id,
                posted_dr.dr_id,
>>>>>>> refactor: add PostDataRequestArgs
            ));
        }

        // save the data request
        let dr_count = DATA_REQUESTS_COUNT.load(deps.storage)?;
        DATA_REQUESTS.save(
            deps.storage,
<<<<<<< HEAD
            args.dr_id.clone(),
=======
            posted_dr.dr_id.clone(),
>>>>>>> refactor: add PostDataRequestArgs
            &DataRequest {
                dr_id: posted_dr.dr_id.clone(),

                dr_binary_id: posted_dr.dr_binary_id,
                tally_binary_id: posted_dr.tally_binary_id,
                dr_inputs: posted_dr.dr_inputs,
                tally_inputs: posted_dr.tally_inputs,
                memo: posted_dr.memo,
                replication_factor: posted_dr.replication_factor,

                gas_price: posted_dr.gas_price,
                gas_limit: posted_dr.gas_limit,

                seda_payload: posted_dr.seda_payload,
                payback_address: posted_dr.payback_address,
                commits: HashMap::new(),
                reveals: HashMap::new(),
            },
        )?;
<<<<<<< HEAD
        DATA_REQUESTS_BY_NONCE.save(deps.storage, dr_count, &args.dr_id)?; // todo wrong nonce
=======
        DATA_REQUESTS_BY_NONCE.save(deps.storage, dr_count, &posted_dr.dr_id)?; // todo wrong nonce
>>>>>>> refactor: add PostDataRequestArgs

        // increment the data request count
        DATA_REQUESTS_COUNT.update(deps.storage, |mut new_dr_id| -> Result<_, ContractError> {
            new_dr_id += 1;
            Ok(new_dr_id)
        })?;

        Ok(Response::new()
            .add_attribute("action", "post_data_request")
<<<<<<< HEAD
            .add_attribute("dr_id", args.dr_id))
=======
            .add_attribute("dr_id", posted_dr.dr_id))
>>>>>>> refactor: add PostDataRequestArgs
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
    use crate::contract::query;
    use crate::helpers::hash_update;
    use crate::msg::PostDataRequestArgs;
    use crate::state::Reveal;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use crate::types::Bytes;
    use crate::types::Commitment;
    use crate::types::Memo;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    use crate::contract::instantiate;
    use crate::msg::GetDataRequestResponse;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn post_data_request() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
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
        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = "hello world".to_string();
        let payback_address: Bytes = Vec::new();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo1: Memo = binary_hash.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo1.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());
        let constructed_dr_id = format!("0x{}", hex::encode(hasher.finalize()));

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id,
            dr_inputs,
            tally_inputs: tally_inputs.clone(),
            memo: memo1,
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload,
            payback_address: payback_address.clone(),
        };
<<<<<<< HEAD
        let msg = ExecuteMsg::PostDataRequest { args };
=======
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
>>>>>>> refactor: add PostDataRequestArgs
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

        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Bytes = Vec::new();
        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();
        let commits: HashMap<String, Commitment> = HashMap::new();
        let reveals: HashMap<String, Reveal> = HashMap::new();
        let chain_id = 31337;
        let nonce = 1;
        let value = "hello world".to_string();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo1: Memo = binary_hash.clone().into_bytes();

        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo1.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id = format!("0x{}", hex::encode(hasher.finalize()));
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
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = 1;
        let payback_address: Bytes = Vec::new();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash1 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo1: Memo = binary_hash1.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo1.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id1 = format!("0x{}", hex::encode(hasher.finalize()));
        let chain_id = 31337;
        let nonce = 1;
        let value = 2;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash2 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo2: Memo = binary_hash2.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo2.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id2 = format!("0x{}", hex::encode(hasher.finalize()));
        let chain_id = 31337;
        let nonce = 1;
        let value = 3;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash3 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo3: Memo = binary_hash3.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo3.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id3 = format!("0x{}", hex::encode(hasher.finalize()));

        let payback_address: Bytes = Vec::new();
        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id1.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),

            memo: memo1,
            replication_factor,
            gas_price,
            gas_limit,
            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
<<<<<<< HEAD
        let msg = ExecuteMsg::PostDataRequest { args };
=======
        // someone posts three data requests
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
>>>>>>> refactor: add PostDataRequestArgs
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
<<<<<<< HEAD
        let msg = ExecuteMsg::PostDataRequest { args };
=======
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
>>>>>>> refactor: add PostDataRequestArgs
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
<<<<<<< HEAD
        let msg = ExecuteMsg::PostDataRequest { args };
=======
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
>>>>>>> refactor: add PostDataRequestArgs
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

        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
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

        let chain_id = 31337;
        let nonce = 1;
        let value = 1;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash1 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo1: Memo = binary_hash1.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo1.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());
        let constructed_dr_id1 = format!("0x{}", hex::encode(hasher.finalize()));

        let chain_id = 31337;
        let nonce = 1;
        let value = 2;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash2 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo2: Memo = binary_hash2.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo2.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());
        let constructed_dr_id2 = format!("0x{}", hex::encode(hasher.finalize()));
        let chain_id = 31337;
        let nonce = 1;
        let value = 3;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash3 = format!("0x{}", hex::encode(hasher.finalize()));
        let memo3: Memo = binary_hash3.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo3.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id3 = format!("0x{}", hex::encode(hasher.finalize()));
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
    fn no_duplicate_dr_ids() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // register dr executor
        let info = mock_info("anyone", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();

        let replication_factor: u16 = 3;

        // set by dr creator
        let gas_price: u128 = 10;
        let gas_limit: u128 = 10;

        // set by relayer and SEDA protocol
        let seda_payload: Bytes = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = 1;
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hash_update(&mut hasher, value);
        let binary_hash1 = format!("0x{}", hex::encode(hasher.clone().finalize()));
        let memo1: Memo = binary_hash1.clone().into_bytes();
        let payback_address: Bytes = Vec::new();
        let mut hasher = Keccak256::new();

        hasher.update(dr_binary_id.clone());
        hasher.update(dr_inputs.clone());
        hasher.update(gas_limit.to_be_bytes().clone());
        hasher.update(gas_price.to_be_bytes().clone());
        hasher.update(memo1.clone());
        hasher.update(payback_address.clone());
        hasher.update(replication_factor.to_be_bytes().clone());
        hasher.update(seda_payload.clone());
        hasher.update(tally_binary_id.clone());
        hasher.update(tally_inputs.clone());

        let constructed_dr_id1 = format!("0x{}", hex::encode(hasher.finalize()));

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
        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: constructed_dr_id1.clone(),
            commitment: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

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

        // can't create a data request with the same id as a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_hash_data_request() {
        let nonce = 1;
        let value = "hello world";
        let chain_id = 31337;
        let wasm_id = "wasm_id".to_string().into_bytes();
        let wasm_args: Vec<Vec<u8>> = vec![
            "arg1".to_string().into_bytes(),
            "arg2".to_string().into_bytes(),
        ];

        let dr_hash = hash_data_request(&nonce, value, &chain_id, wasm_id.as_slice(), &wasm_args);

        println!("dr_hash: {}", dr_hash);
    }
}