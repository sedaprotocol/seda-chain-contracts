#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use common::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use common::types::Hash;

pub mod data_request_results {

    use std::ops::Deref;

    use common::error::ContractError::{
        self, AlreadyCommitted, AlreadyRevealed, IneligibleExecutor, NotCommitted, RevealMismatch,
        RevealNotStarted, RevealStarted,
    };
    use cosmwasm_std::{Addr, Env, Event, QuerierWrapper};
    use sha3::{Digest, Keccak256};

    use common::msg::{
        GetCommittedDataResultsResponse, GetCommittedExecutorsResponse,
        GetResolvedDataResultResponse, GetRevealedDataResultsResponse, QuerySeedResponse,
        SpecialQueryMsg, SpecialQueryWrapper,
    };
    use common::state::{DataResult, Reveal};
    use common::types::Bytes;

    use crate::contract::CONTRACT_VERSION;
    use crate::state::DATA_REQUESTS_POOL;
    use crate::utils::hash_to_string;
    use crate::{
        state::DATA_RESULTS,
        utils::{check_eligibility, hash_data_result, validate_sender},
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
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        if dr.commits.contains_key(&sender.to_string()) {
            return Err(AlreadyCommitted);
        }

        // error if reveal stage has started (replication factor reached)
        if u16::try_from(dr.commits.len()).unwrap() >= dr.replication_factor {
            return Err(RevealStarted);
        }

        // add the commitment to the data request
        dr.commits.insert(sender.to_string(), commitment);

        DATA_REQUESTS_POOL.update(deps.storage, dr_id, &dr)?;

        Ok(Response::new()
            .add_attribute("action", "commit_data_result")
            .add_event(Event::new("seda-commitment").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(dr_id)),
                ("executor", sender.as_str()),
                ("commitment", &hash_to_string(commitment)),
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
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
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

        let committed_dr_result = *committed_dr_results.get(&sender.to_string()).unwrap();

        let calculated_dr_result = compute_hash(&reveal.reveal, &reveal.salt);
        if calculated_dr_result != committed_dr_result {
            return Err(RevealMismatch);
        }

        dr.reveals.insert(sender.to_string(), reveal.clone());

        DATA_REQUESTS_POOL.update(deps.storage, dr_id, &dr)?;

        let mut response = Response::new()
            .add_attribute("action", "reveal_data_result")
            .add_event(Event::new("seda-reveal").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(dr_id)),
                ("executor", sender.as_str()),
                ("reveal", serde_json::to_string(&reveal).unwrap().as_str()),
            ]));

        // if total reveals equals replication factor, resolve the DR
        // TODO: this needs to be separated out in a tally function once the module is implemented
        if u16::try_from(dr.reveals.len()).unwrap() == dr.replication_factor {
            let block_height: u64 = env.block.height;
            let exit_code: u8 = 0;
            let result: Bytes = reveal.reveal.as_bytes().to_vec();

            let payback_address: Bytes = dr.payback_address.clone();
            let seda_payload: Bytes = dr.seda_payload.clone();

            // save the data result
            let result_id = hash_data_result(&dr, block_height, exit_code, &result);
            let dr_result = DataResult {
                version: dr.version,
                result_id,
                dr_id,
                block_height,
                exit_code,
                result: result.clone(),
                payback_address: payback_address.clone(),
                seda_payload: seda_payload.clone(),
            };
            DATA_RESULTS.save(deps.storage, dr_id, &dr_result)?;

            // remove from the pool
            DATA_REQUESTS_POOL.remove(deps.storage, dr_id)?;

            response = response.add_event(Event::new("seda-data-result").add_attributes([
                ("version", CONTRACT_VERSION),
                ("result_id", &hash_to_string(result_id)),
                ("dr_id", &hash_to_string(dr_id)),
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

        Ok(response)
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_committed_data_result(
        deps: Deps,
        dr_id: Hash,
        executor: Addr,
    ) -> StdResult<GetCommittedDataResultResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
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
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        Ok(GetCommittedDataResultsResponse { value: dr.commits })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_result(
        deps: Deps,
        dr_id: Hash,
        executor: Addr,
    ) -> StdResult<GetRevealedDataResultResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
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
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
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

    /// Returns a vector of committed executors
    pub fn get_committed_executors(
        deps: Deps,
        dr_id: Hash,
    ) -> StdResult<GetCommittedExecutorsResponse> {
        let mut executors = Vec::new();
        for key in DATA_REQUESTS_POOL.load(deps.storage, dr_id)?.commits.keys() {
            executors.push(key.clone())
        }
        Ok(GetCommittedExecutorsResponse { value: executors })
    }

    pub fn get_seed(deps: Deps) -> StdResult<QuerySeedResponse> {
        let req = SpecialQueryWrapper {
            query_data: SpecialQueryMsg::QuerySeedRequest {},
        }
        .into();
        let wrapper: QuerierWrapper<'_, SpecialQueryWrapper> =
            QuerierWrapper::new(deps.querier.deref());
        let response: QuerySeedResponse = wrapper.query(&req).unwrap();
        Ok(QuerySeedResponse {
            block_height: response.block_height,
            seed: response.seed,
        })
    }

    /// Computes hash given a reveal and salt
    fn compute_hash(reveal: &str, salt: &str) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(reveal.as_bytes());
        hasher.update(salt.as_bytes());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod data_request_result_tests {
    use crate::contract::execute;
    use crate::helpers::instantiate_dr_contract;
    use crate::utils::string_to_hash;
    use common::msg::DataRequestsExecuteMsg;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    #[should_panic(expected = "NotProxy")]
    fn only_proxy_can_pass_caller() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(2, "token"));

        // instantiate contract
        instantiate_dr_contract(deps.as_mut(), info).unwrap();

        // try commiting a data result from a non-proxy (doesn't matter if it's eligible or not since sender validation comes first)
        let msg = DataRequestsExecuteMsg::CommitDataResult {
            dr_id: string_to_hash("dr_id"),
            commitment: string_to_hash("commitment"),
            sender: Some("someone".to_string()),
        };
        let info = mock_info("anyone", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
