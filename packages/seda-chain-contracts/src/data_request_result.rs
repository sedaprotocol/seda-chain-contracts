#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use crate::state::DATA_REQUESTS;
use crate::types::Hash;

use crate::ContractError;

pub mod data_request_results {

    use cosmwasm_std::Addr;
    // use hex_literal::hex;
    use sha3::{Digest, Keccak256};

    use crate::{
        msg::{GetCommittedExecutorsResponse, GetIdsResponse},
        state::{Reveal, DATA_RESULTS},
        utils::check_eligibility,
    };

    use super::*;

    /// Posts a data result of a data request with an attached hash of the answer and salt.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn commit_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        commitment: Hash,
    ) -> Result<Response, ContractError> {
        assert!(check_eligibility(&deps, info.sender.clone())?);
        // find the data request from the pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS.load(deps.storage, dr_id.clone())?;
        if dr.commits.contains_key(&info.sender) {
            panic!("sender already committed before");
        }
        dr.commits.insert(info.sender, commitment.clone());

        DATA_REQUESTS.save(deps.storage, dr_id.clone(), &dr)?;

        Ok(Response::new()
            .add_attribute("action", "commit_result")
            .add_attribute("dr_id", dr_id)
            .add_attribute("result", commitment))
    }

    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn reveal_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        reveal: Reveal,
    ) -> Result<Response, ContractError> {
        assert!(
            check_eligibility(&deps, info.sender.clone())?,
            "sender is not an eligible dr executor"
        );

        // find the data request from the committed pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS.load(deps.storage, dr_id.clone())?;
        let committed_dr_results = dr.clone().commits;

        assert!(
            u16::try_from(committed_dr_results.len()).unwrap() >= dr.replication_factor,
            "Revealing didn't start yet"
        );
        if !committed_dr_results.contains_key(&info.sender) {
            panic!("executor hasn't committed");
        }

        let committed_dr_result = committed_dr_results.get(&info.sender).unwrap().clone();

        let calculated_dr_result = compute_hash(&reveal.reveal) + &compute_hash(&reveal.salt);
        assert_eq!(
            calculated_dr_result, committed_dr_result,
            "committed result doesn't match revealed result"
        );

        if dr.reveals.contains_key(&info.sender) {
            panic!("sender already revealed");
        }

        dr.reveals.insert(info.sender, reveal.clone());

        // save the data result in REVEALED_DATA_RESULTS pool
        DATA_REQUESTS.save(deps.storage, dr_id.clone(), &dr)?;

        Ok(Response::new()
            .add_attribute("action", "commit_result")
            .add_attribute("dr_id", dr_id)
            .add_attribute("reveal", reveal.reveal))
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_committed_data_result(
        deps: Deps,
        dr_id: Hash,
        executor: Addr,
    ) -> StdResult<GetCommittedDataResultResponse> {
        let dr = DATA_REQUESTS.load(deps.storage, dr_id)?;
        let commitment = dr.commits.get(&executor);
        Ok(GetCommittedDataResultResponse {
            value: commitment.cloned(),
        })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_result(
        deps: Deps,
        dr_id: Hash,
        executor: Addr,
    ) -> StdResult<GetRevealedDataResultResponse> {
        let dr = DATA_REQUESTS.load(deps.storage, dr_id)?;
        let reveal = dr.reveals.get(&executor);
        Ok(GetRevealedDataResultResponse {
            value: reveal.cloned(),
        })
    }

    /// Returns a vector of committed data requests ids, if it exists.
    pub fn get_drs_ids(deps: Deps) -> StdResult<GetIdsResponse> {
        let mut ids = Vec::new();
        for (_, key) in DATA_REQUESTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .enumerate()
        {
            ids.push(key?)
        }
        Ok(GetIdsResponse { value: ids })
    }

    /// Returns a vector of committed data requests ids, if it exists.
    pub fn get_results_ids(deps: Deps) -> StdResult<GetIdsResponse> {
        let mut ids = Vec::new();
        for (_, key) in DATA_RESULTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .enumerate()
        {
            ids.push(key?)
        }
        Ok(GetIdsResponse { value: ids })
    }

    /// Returns a vector of committed data requests ids, if it exists.
    pub fn get_committed_executors(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetCommittedExecutorsResponse> {
        let mut executors = Vec::new();
        for key in DATA_REQUESTS.load(deps.storage, dr_id)?.commits.keys() {
            executors.push(key.clone())
        }
        Ok(GetCommittedExecutorsResponse { value: executors })
    }

    /// Returns a vector of revealed data requests ids, if it exists.

    fn compute_hash(input_data: &str) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(input_data.as_bytes());
        let digest = hasher.finalize();
        format!("0x{}", hex::encode(digest))
    }
}

#[cfg(test)]
mod dr_result_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::query;
    use crate::msg::PostDataRequestArgs;
    use crate::helpers::hash_update;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use crate::types::Input;
    use crate::types::Memo;
    use crate::types::PayloadItem;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use cosmwasm_std::{coins, from_binary};
    use sha3::{Digest, Keccak256};

    use crate::contract::instantiate;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};
    use crate::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};

    #[test]
    fn commit_result() {
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

        // can't post a data result for a data request that doesn't exist
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
            commitment: "dr 0 result".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Vec<Input> = Vec::new();
        let tally_inputs: Vec<Input> = Vec::new();
        let replication_factor: u16 = 3;

        let gas_price: u128 = 0;
        let gas_limit: u128 = 0;

        let payload: Vec<PayloadItem> = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = "hello world".to_string();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo: Memo = binary_hash.clone().into_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(memo.clone());
    
        let constructed_dr_id = format!("0x{}", hex::encode(hasher.finalize()));
        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            dr_id: constructed_dr_id.clone(),

            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id,
            dr_inputs,
            tally_inputs,
            memo,
            replication_factor,

            gas_price,
            gas_limit,

            payload,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // can fetch it via `get_data_requests_from_pool`
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: None,
            },
        );
        let value: GetDataRequestsFromPoolResponse = from_binary(&res.unwrap()).unwrap();
        assert_eq!(value.value.len(), 1);
        let executor: Addr = info.sender.clone();
        // data result with id 0x66... does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommittedDataResult {
                dr_id: constructed_dr_id.clone(),
                executor: executor.clone(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: constructed_dr_id.clone(),
            commitment: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // should be able to fetch data result with id 0x66...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommittedDataResult {
                dr_id: constructed_dr_id.clone(),
                executor: executor.clone(),
            },
        )
        .unwrap();
        let value: GetCommittedDataResultResponse = from_binary(&res).unwrap();

        assert_eq!(Some("dr 0 result".to_string()), value.value);

        // can no longer fetch the first via `get_data_requests_from_pool`, only the second
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestsFromPool {
                position: None,
                limit: None,
            },
        );
        let value: GetDataRequestsFromPoolResponse = from_binary(&res.unwrap()).unwrap();
        assert_eq!(value.value.len(), 0);
    }

    #[test]
    #[should_panic]

    fn ineligible_post_data_result() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Vec<Input> = Vec::new();
        let tally_inputs: Vec<Input> = Vec::new();
        let replication_factor: u16 = 3;

        let gas_price: u128 = 0;
        let gas_limit: u128 = 0;

        let payload: Vec<PayloadItem> = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = "hello world".to_string();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, chain_id);
        hash_update(&mut hasher, nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo: Memo = binary_hash.clone().into_bytes();

        // set arguments for post_data_request
        // TODO: move this and duplicates to a helper function
        let wasm_id = "wasm_id".to_string().into_bytes();
        let wasm_args: Vec<Vec<u8>> = vec![
            "arg1".to_string().into_bytes(),
            "arg2".to_string().into_bytes(),
        ];

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            dr_id: binary_hash.clone(),
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id,
            dr_inputs,
            tally_inputs,
            memo,
            replication_factor,

            gas_price,
            gas_limit,

            payload,
        };
        let msg = ExecuteMsg::PostDataRequest { args };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // ineligible shouldn't be able to post a data result
        let info = mock_info("ineligible", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: binary_hash.clone(),
            commitment: "dr 0 result".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
