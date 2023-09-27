#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::state::DATA_REQUESTS;
use common::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use common::types::Hash;

use crate::error::ContractError;

pub mod data_request_results {

    use cosmwasm_std::{Addr, Env, Event};
    use sha3::{Digest, Keccak256};

    use common::msg::{
        GetCommittedDataResultsResponse, GetCommittedExecutorsResponse, GetIdsResponse,
        GetResolvedDataResultResponse, GetRevealedDataResultsResponse,
    };
    use common::state::{DataResult, Reveal};
    use common::types::Bytes;

    use crate::contract::CONTRACT_VERSION;
    use crate::{
        state::DATA_RESULTS,
        utils::{check_eligibility, hash_data_result, validate_sender},
        ContractError::{
            AlreadyCommitted, AlreadyRevealed, IneligibleExecutor, NotCommitted, RevealMismatch,
            RevealNotStarted,
        },
    };

    use super::*;

    /// Posts a data result of a data request with an attached hash of the answer and salt.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn commit_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        commitment: Hash,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;
        if !check_eligibility(&deps, sender.clone())? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS.load(deps.storage, dr_id.clone())?;
        if dr.commits.contains_key(&sender.to_string()) {
            return Err(AlreadyCommitted);
        }
        dr.commits.insert(sender.to_string(), commitment.clone());

        DATA_REQUESTS.save(deps.storage, dr_id.clone(), &dr)?;

        Ok(Response::new()
            .add_attribute("action", "commit_data_result")
            .add_event(Event::new("seda-commitment").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", dr_id.as_str()),
                ("executor", sender.as_str()),
                ("commitment", commitment.as_str()),
            ])))
    }

    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn reveal_result(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        dr_id: Hash,
        reveal: Reveal,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;
        if !check_eligibility(&deps, sender.clone())? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the committed pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS.load(deps.storage, dr_id.clone())?;
        let committed_dr_results = dr.clone().commits;

        if u16::try_from(committed_dr_results.len()).unwrap() < dr.replication_factor {
            return Err(RevealNotStarted);
        }
        if !committed_dr_results.contains_key(&sender.to_string()) {
            return Err(NotCommitted);
        }
        if dr.reveals.contains_key(&sender.to_string()) {
            return Err(AlreadyRevealed);
        }

        let committed_dr_result = committed_dr_results
            .get(&sender.to_string())
            .unwrap()
            .clone();

        let calculated_dr_result = compute_hash(&reveal.reveal, &reveal.salt);
        if calculated_dr_result != committed_dr_result {
            return Err(RevealMismatch);
        }

        dr.reveals.insert(sender.to_string(), reveal.clone());

        DATA_REQUESTS.save(deps.storage, dr_id.clone(), &dr)?;

        let mut events = vec![Event::new("seda-reveal").add_attributes([
            ("version", CONTRACT_VERSION),
            ("dr_id", dr_id.as_str()),
            ("executor", sender.as_str()),
            ("reveal", serde_json::to_string(&reveal).unwrap().as_str()),
        ])];

        if u16::try_from(dr.reveals.len()).unwrap() == dr.replication_factor {
            let block_height: u64 = env.block.height;
            let exit_code: u8 = 0;
            let result: Bytes = reveal.reveal.as_bytes().to_vec();

            let payback_address: Bytes = dr.payback_address.clone();
            let seda_payload: Bytes = dr.seda_payload.clone();

            let result_id = hash_data_result(&dr, block_height, exit_code, &result);

            let dr_result = DataResult {
                result_id: result_id.clone(),
                dr_id: dr_id.clone(),
                block_height,
                exit_code,
                result: result.clone(),
                payback_address: payback_address.clone(),
                seda_payload: seda_payload.clone(),
            };
            DATA_RESULTS.save(deps.storage, dr_id.clone(), &dr_result)?;
            DATA_REQUESTS.remove(deps.storage, dr_id.clone());

            events.push(Event::new("seda-data-result").add_attributes([
                ("version", CONTRACT_VERSION),
                ("result_id", &result_id),
                ("dr_id", &dr_id),
                ("block_height", &block_height.to_string()),
                ("exit_code", &exit_code.to_string()),
                ("result", &serde_json::to_string(&result).unwrap()),
                (
                    "payback_address",
                    &serde_json::to_string(&payback_address).unwrap(),
                ),
                (
                    "seda_payload",
                    &serde_json::to_string(&seda_payload).unwrap(),
                ),
            ]));
        }

        Ok(Response::new()
            .add_attribute("action", "reveal_data_result")
            .add_events(events))
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
