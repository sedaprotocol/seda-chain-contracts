#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use crate::state::DATA_REQUESTS;
use crate::types::Hash;

use crate::ContractError;

pub mod data_request_results {

    use cosmwasm_std::{Addr, Env};
    use sha3::{Digest, Keccak256};

    use crate::{
        msg::{
            GetCommittedDataResultsResponse, GetCommittedExecutorsResponse, GetIdsResponse,
            GetResolvedDataResultResponse, GetRevealedDataResultsResponse,
        },
        state::{DataResult, Reveal, DATA_RESULTS},
        types::Bytes,
        utils::{check_eligibility, compute_result_hash},
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
        if dr.commits.contains_key(&info.sender.to_string()) {
            panic!("sender already committed before");
        }
        dr.commits
            .insert(info.sender.to_string(), commitment.clone());

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
        env: Env,
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
        if !committed_dr_results.contains_key(&info.sender.to_string()) {
            panic!("executor hasn't committed");
        }
        if dr.reveals.contains_key(&info.sender.to_string()) {
            panic!("sender already revealed");
        }

        let committed_dr_result = committed_dr_results
            .get(&info.sender.to_string())
            .unwrap()
            .clone();

        let calculated_dr_result = compute_hash(&reveal.reveal, &reveal.salt);
        assert_eq!(
            calculated_dr_result, committed_dr_result,
            "committed result doesn't match revealed result"
        );

        dr.reveals.insert(info.sender.to_string(), reveal.clone());

        DATA_REQUESTS.save(deps.storage, dr_id.clone(), &dr)?;

        if u16::try_from(dr.reveals.len()).unwrap() == dr.replication_factor {
            let block_height: u64 = env.block.height;
            let exit_code: u8 = 0;
            let result: Bytes = reveal.reveal.as_bytes().to_vec();

            let payback_address: Bytes = dr.payback_address.clone();
            let seda_payload: Bytes = dr.seda_payload.clone();

            let result_id = compute_result_hash(&dr, block_height, exit_code, &result);

            let dr_result = DataResult {
                result_id,
                dr_id: dr_id.clone(),
                block_height,
                exit_code,
                result,
                payback_address,
                seda_payload,
            };
            DATA_RESULTS.save(deps.storage, dr_id.clone(), &dr_result)?;
            DATA_REQUESTS.remove(deps.storage, dr_id.clone());
        }

        Ok(Response::new()
            .add_attribute("action", "reveal_result")
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
        let commitment = dr.commits.get(&executor.to_string());
        Ok(GetCommittedDataResultResponse {
            value: commitment.cloned(),
        })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_committed_data_results(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetCommittedDataResultsResponse> {
        let dr = DATA_REQUESTS.load(deps.storage, dr_id)?;
        Ok(GetCommittedDataResultsResponse { value: dr.commits })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_result(
        deps: Deps,
        dr_id: Hash,
        executor: Addr,
    ) -> StdResult<GetRevealedDataResultResponse> {
        let dr = DATA_REQUESTS.load(deps.storage, dr_id)?;
        let reveal = dr.reveals.get(&executor.to_string());
        Ok(GetRevealedDataResultResponse {
            value: reveal.cloned(),
        })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_results(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetRevealedDataResultsResponse> {
        let dr = DATA_REQUESTS.load(deps.storage, dr_id)?;
        Ok(GetRevealedDataResultsResponse { value: dr.reveals })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_resolved_data_result(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetResolvedDataResultResponse> {
        let result = DATA_RESULTS.load(deps.storage, dr_id)?;
        Ok(GetResolvedDataResultResponse { value: result })
    }

    /// Returns a vector of data requests ids
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

    /// Returns a vector of data results ids
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

    /// Returns a vector of committed executors
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

    fn compute_hash(reveal: &str, salt: &str) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(reveal.as_bytes());
        hasher.update(salt.as_bytes());
        let digest = hasher.finalize();
        format!("0x{}", hex::encode(digest))
    }
}

#[cfg(test)]
mod dr_result_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::query;
    use crate::msg::GetResolvedDataResultResponse;
    use crate::msg::PostDataRequestArgs;
    use crate::state::DataRequestInputs;
    use crate::state::Reveal;
    use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
    use crate::types::Bytes;
    use crate::types::Memo;
    use crate::utils::hash_data_request;
    use crate::utils::hash_update;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use cosmwasm_std::{coins, from_binary};
    use sha3::{Digest, Keccak256};

    use crate::contract::instantiate;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};
    use crate::msg::{GetDataRequestResponse, GetDataRequestsFromPoolResponse};

    #[test]
    fn commit_reveal_result() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // register dr executor
        let info = mock_info("executor1", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address1".to_string()),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // register dr executor
        let info = mock_info("executor2", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address2".to_string()),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor_is_eligible: bool = ELIGIBLE_DATA_REQUEST_EXECUTORS
            .load(&deps.storage, info.sender.clone())
            .unwrap();
        assert!(executor_is_eligible);

        // can't post a data result for a data request that doesn't exist
        let info = mock_info("executor1", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: "0x7e059b547de461457d49cd4b229c5cd172a6ac8063738068b932e26c3868e4ae".to_string(),
            commitment: "dr 0 result".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        let dr_binary_id: Hash = "".to_string();
        let tally_binary_id: Hash = "".to_string();
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();
        let replication_factor: u16 = 2;

        let gas_price: u128 = 0;
        let gas_limit: u128 = 0;

        let seda_payload: Bytes = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = "ETH/USD".to_string();
        let payback_address: Bytes = Vec::new();

        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, &chain_id);
        hash_update(&mut hasher, &nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo: Memo = binary_hash.clone().into_bytes();
        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let constructed_dr_id = hash_data_request(dr_inputs1);

        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: constructed_dr_id.clone(),

            dr_binary_id: dr_binary_id.clone(),
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
        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
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

        let reveal = "2000";
        let salt = "executor1";

        let mut hasher = Keccak256::new();
        hasher.update(reveal.as_bytes());
        hasher.update(salt.as_bytes());
        let digest = hasher.finalize();
        let commitment1 = format!("0x{}", hex::encode(digest));

        // executor1 commits a data result
        let info = mock_info("executor1", &coins(2, "token"));

        let msg = ExecuteMsg::CommitDataResult {
            dr_id: constructed_dr_id.clone(),
            commitment: commitment1,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let reveal = "2200";
        let salt = "executor2";
        let mut hasher = Keccak256::new();
        hasher.update(reveal.as_bytes());
        hasher.update(salt.as_bytes());
        let digest = hasher.finalize();
        let commitment2 = format!("0x{}", hex::encode(digest));

        // executor2 commits a data result
        let info = mock_info("executor2", &coins(2, "token"));
        let msg = ExecuteMsg::CommitDataResult {
            dr_id: constructed_dr_id.clone(),
            commitment: commitment2.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let executor2: Addr = info.sender.clone();

        // should be able to fetch data result with id 0x66...
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommittedDataResult {
                dr_id: constructed_dr_id.clone(),
                executor: executor2.clone(),
            },
        )
        .unwrap();
        let value: GetCommittedDataResultResponse = from_binary(&res).unwrap();

        assert_eq!(Some(commitment2.clone()), value.value);
        let reveal1 = Reveal {
            reveal: "2000".to_string(),
            salt: "executor1".to_string(),
        };
        let info = mock_info("executor1", &coins(2, "token"));
        let executor1 = info.sender.clone();
        let msg = ExecuteMsg::RevealDataResult {
            dr_id: constructed_dr_id.clone(),
            reveal: reveal1.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRevealedDataResult {
                dr_id: constructed_dr_id.clone(),
                executor: executor1.clone(),
            },
        )
        .unwrap();
        let value: GetRevealedDataResultResponse = from_binary(&res).unwrap();

        assert_eq!(Some(reveal1.clone()), value.value);
        let reveal2 = Reveal {
            reveal: "2200".to_string(),
            salt: "executor2".to_string(),
        };

        let info = mock_info("executor2", &coins(2, "token"));
        let msg = ExecuteMsg::RevealDataResult {
            dr_id: constructed_dr_id.clone(),
            reveal: reveal2.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetResolvedDataResult {
                dr_id: constructed_dr_id.clone(),
            },
        )
        .unwrap();
        let value: GetResolvedDataResultResponse = from_binary(&res).unwrap();
        assert_eq!(reveal2.reveal.as_bytes().to_vec(), value.value.result);
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
        let dr_inputs: Bytes = Vec::new();
        let tally_inputs: Bytes = Vec::new();
        let replication_factor: u16 = 3;

        let gas_price: u128 = 0;
        let gas_limit: u128 = 0;

        let seda_payload: Bytes = Vec::new();

        let chain_id = 31337;
        let nonce = 1;
        let value = "hello world".to_string();
        let mut hasher = Keccak256::new();
        hash_update(&mut hasher, &chain_id);
        hash_update(&mut hasher, &nonce);
        hasher.update(value);
        let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));
        let memo: Memo = binary_hash.clone().into_bytes();
        let payback_address: Bytes = Vec::new();

        let dr_inputs1 = DataRequestInputs {
            dr_binary_id: dr_binary_id.clone(),
            tally_binary_id: tally_binary_id.clone(),
            dr_inputs: dr_inputs.clone(),
            tally_inputs: tally_inputs.clone(),
            memo: memo.clone(),
            replication_factor,

            gas_price,
            gas_limit,

            seda_payload: seda_payload.clone(),
            payback_address: payback_address.clone(),
        };
        let dr_id: String = hash_data_request(dr_inputs1);
        let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
            dr_id: dr_id.clone(),

            dr_binary_id: dr_binary_id.clone(),
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
        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest { posted_dr };
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
