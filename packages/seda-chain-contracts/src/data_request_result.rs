#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use crate::state::{COMMITTED_DATA_RESULTS, DATA_REQUESTS_POOL};
use crate::types::Hash;

use crate::state::CommittedDataResult;
use crate::ContractError;

pub mod data_request_results {

    // use hex_literal::hex;
    use sha3::{Digest, Sha3_256};

    use crate::{
        consts::COMMITS_THRESHOLD,
        msg::GetDataResultsIdsResponse,
        state::{RevealedDataResult, REVEALED_DATA_RESULTS},
        utils::check_eligibility,
    };

    use super::*;

    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn commit_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        result: Hash,
    ) -> Result<Response, ContractError> {
        assert!(check_eligibility(&deps, info.sender.clone())?);
        // find the data request from the pool (if it exists, otherwise error)
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id.clone())?;
        let dr_result = CommittedDataResult {
            dr_id: dr.dr_id,
            nonce: dr.nonce,
            value: dr.value,
            result: result.clone(),
            chain_id: dr.chain_id,
            executor: info.sender,
        };
        let mut committed_drs = Vec::new();
        if COMMITTED_DATA_RESULTS.has(deps.storage, dr_id.clone()) {
            committed_drs = COMMITTED_DATA_RESULTS.load(deps.storage, dr_id.clone())?;
        }
        committed_drs.push(dr_result);

        // save the data result then remove it from the pool
        COMMITTED_DATA_RESULTS.save(deps.storage, dr_id.clone(), &committed_drs)?;
        DATA_REQUESTS_POOL.remove(deps.storage, dr_id.clone());

        Ok(Response::new()
            .add_attribute("action", "commit_result")
            .add_attribute("dr_id", dr_id)
            .add_attribute("result", result))
    }

    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn reveal_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        answer: String,
        salt: String,
    ) -> Result<Response, ContractError> {
        assert!(
            check_eligibility(&deps, info.sender.clone())?,
            "sender is not an eligible dr executor"
        );

        // find the data request from the committed pool (if it exists, otherwise error)
        let mut committed_dr_results = COMMITTED_DATA_RESULTS.load(deps.storage, dr_id.clone())?;
        assert!(
            u128::try_from(committed_dr_results.len()).unwrap() >= COMMITS_THRESHOLD,
            "Revealing didn't start yet"
        );
        let mut committed_dr: Option<CommittedDataResult> = None;
        for (index, committed) in committed_dr_results.clone().iter_mut().enumerate() {
            if committed.executor == info.sender.clone() {
                committed_dr = Some(committed.clone());
                committed_dr_results.remove(index);
                COMMITTED_DATA_RESULTS.save(deps.storage, dr_id.clone(), &committed_dr_results)?;
            } else {
                panic!("executor hasn't committed an answer");
            }
        }
        let committed_dr = committed_dr.unwrap();

        let calculated_dr_result = compute_hash(&answer) + &compute_hash(&salt);
        assert_eq!(
            calculated_dr_result, committed_dr.result,
            "committed result doesn't match revealed result"
        );
        let dr_result = RevealedDataResult {
            dr_id: committed_dr.dr_id,
            nonce: committed_dr.nonce,
            value: committed_dr.value,
            chain_id: committed_dr.chain_id,
            executor: info.sender,
            answer: answer.clone(),
            salt,
        };

        let mut revealed_drs = Vec::new();
        if REVEALED_DATA_RESULTS.has(deps.storage, dr_id.clone()) {
            revealed_drs = REVEALED_DATA_RESULTS.load(deps.storage, dr_id.clone())?;
        }
        revealed_drs.push(dr_result);

        // save the data result then remove it from the pool
        REVEALED_DATA_RESULTS.save(deps.storage, dr_id.clone(), &revealed_drs)?;
        DATA_REQUESTS_POOL.remove(deps.storage, dr_id.clone());

        Ok(Response::new()
            .add_attribute("action", "commit_result")
            .add_attribute("dr_id", dr_id)
            .add_attribute("answer", answer))
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_committed_data_result(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetCommittedDataResultResponse> {
        let dr = COMMITTED_DATA_RESULTS.may_load(deps.storage, dr_id)?;
        Ok(GetCommittedDataResultResponse { value: dr })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_result(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetRevealedDataResultResponse> {
        let dr = REVEALED_DATA_RESULTS.may_load(deps.storage, dr_id)?;
        Ok(GetRevealedDataResultResponse { value: dr })
    }

    /// Returns a vector of committed data requests ids, if it exists.
    pub fn get_committed_data_results_ids(deps: Deps) -> StdResult<GetDataResultsIdsResponse> {
        let mut dr_ids = Vec::new();
        for (_, key) in COMMITTED_DATA_RESULTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .enumerate()
        {
            dr_ids.push(key?)
        }
        Ok(GetDataResultsIdsResponse { value: dr_ids })
    }

    /// Returns a vector of revealed data requests ids, if it exists.
    pub fn get_revealed_data_results_ids(deps: Deps) -> StdResult<GetDataResultsIdsResponse> {
        let mut dr_ids = Vec::new();
        for (_, key) in REVEALED_DATA_RESULTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .enumerate()
        {
            dr_ids.push(key?)
        }
        Ok(GetDataResultsIdsResponse { value: dr_ids })
    }

    fn compute_hash(input_data: &str) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(input_data.as_bytes());
        let digest = hasher.finalize();
        format!("{:x}", digest)
    }
}

#[cfg(test)]
mod dr_result_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::query;
    use crate::msg::PostDataRequestArgs;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

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
        let msg = ExecuteMsg::PostDataResult {
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            result: "dr 0 result".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        // set arguments for post_data_request
        // TODO: move this and duplicates to a helper function
        let wasm_id = "wasm_id".to_string().into_bytes();
        let wasm_args: Vec<Vec<u8>> = vec![
            "arg1".to_string().into_bytes(),
            "arg2".to_string().into_bytes(),
        ];

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let args = PostDataRequestArgs {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            wasm_id: wasm_id.clone(),
            wasm_args: wasm_args.clone(),
        };
        let msg = ExecuteMsg::PostDataRequest { args };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

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

        // data result with id 0x66... does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommittedDataResult {
                dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            result: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // should be able to fetch data result with id 0x66...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommittedDataResult {
                dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae"
                    .to_string(),
            },
        )
        .unwrap();
        let value: GetCommittedDataResultResponse = from_binary(&res).unwrap();
        let mut res = Vec::new();
        res.push(CommittedDataResult {
            value: "hello world".to_string(),
            nonce: 1,
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
            result: "dr 0 result".to_string(),
            chain_id: 31337,
            executor: info.clone().sender.clone(),
        });
        assert_eq!(Some(res), value.value);

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

        // set arguments for post_data_request
        // TODO: move this and duplicates to a helper function
        let wasm_id = "wasm_id".to_string().into_bytes();
        let wasm_args: Vec<Vec<u8>> = vec![
            "arg1".to_string().into_bytes(),
            "arg2".to_string().into_bytes(),
        ];

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let args = PostDataRequestArgs {
            value: "hello world".to_string(),
            chain_id: 31337,
            nonce: 1,
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            wasm_id: wasm_id,
            wasm_args: wasm_args,
        };
        let msg = ExecuteMsg::PostDataRequest { args };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // ineligible shouldn't be able to post a data result
        let info = mock_info("ineligible", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: "0x69a6e26b4d65f5b3010254a0aae2bf1bc8dccb4ddd27399c580eb771446e719f".to_string(),
            result: "dr 0 result".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
