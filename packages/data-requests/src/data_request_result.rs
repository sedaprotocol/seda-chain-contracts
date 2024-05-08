#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use common::msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse};
use common::types::Hash;

pub mod data_request_results {

    use common::crypto::recover_pubkey;
    use common::error::ContractError::{
        self, AlreadyCommitted, AlreadyRevealed, IneligibleExecutor, NotCommitted, RevealMismatch,
        RevealNotStarted, RevealStarted,
    };
    use cosmwasm_std::{Env, Event};
    use sha3::{Digest, Keccak256};

    use common::msg::{
        GetCommittedDataResultsResponse, GetCommittedExecutorsResponse,
        GetResolvedDataResultResponse, GetRevealedDataResultsResponse,
    };
    use common::state::{DataResult, RevealBody};
    use common::types::{Bytes, Secpk256k1PublicKey, Signature};

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
        public_key: Secpk256k1PublicKey,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        if !check_eligibility(&deps, public_key.clone())? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let public_key_str = hex::encode(public_key);
        if dr.commits.contains_key(&public_key_str) {
            return Err(AlreadyCommitted);
        }

        // error if reveal stage has started (replication factor reached)
        if u16::try_from(dr.commits.len()).unwrap() >= dr.replication_factor {
            return Err(RevealStarted);
        }

        // add the commitment to the data request
        dr.commits.insert(public_key_str, commitment);

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
        reveal_body: RevealBody,
        signature: Signature,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // compute hash of reveal body
        let reveal_body_hash = compute_hash(reveal_body.clone());

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(reveal_body_hash, signature)?;
        if !check_eligibility(&deps, public_key.clone())? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the committed pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;

        // error if reveal phase for this DR has not started (i.e. replication factor is not met)
        let committed_dr_results = dr.clone().commits;
        if u16::try_from(committed_dr_results.len()).unwrap() < dr.replication_factor {
            return Err(RevealNotStarted);
        }

        // error if data request executor has not submitted a commitment
        let public_key_str = hex::encode(public_key);
        if !committed_dr_results.contains_key(&public_key_str) {
            return Err(NotCommitted);
        }

        // error if data request executor has already submitted a reveal
        if dr.reveals.contains_key(&public_key_str) {
            return Err(AlreadyRevealed);
        }

        // find the commitment of this data request executor
        let committed_dr_result = *committed_dr_results.get(&public_key_str).unwrap();

        // error if the commitment hash does not match the reveal
        if reveal_body_hash != committed_dr_result {
            return Err(RevealMismatch);
        }

        // add the reveal to the data request state
        dr.reveals.insert(public_key_str, reveal_body.clone());
        DATA_REQUESTS_POOL.update(deps.storage, dr_id, &dr)?;

        let mut response = Response::new()
            .add_attribute("action", "reveal_data_result")
            .add_event(Event::new("seda-reveal").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(dr_id)),
                ("executor", sender.as_str()),
                (
                    "reveal",
                    serde_json::to_string(&reveal_body).unwrap().as_str(),
                ),
            ]));

        // if total reveals equals replication factor, resolve the DR
        // TODO: this needs to be separated out in a tally function once the module is implemented
        if u16::try_from(dr.reveals.len()).unwrap() == dr.replication_factor {
            let block_height: u64 = env.block.height;
            let exit_code: u8 = 0; // TODO: get this from the tally module
            let gas_used = reveal_body.clone().gas_used;
            let result: Bytes = reveal_body.clone().reveal.to_vec();

            let payback_address: Bytes = dr.payback_address.clone();
            let seda_payload: Bytes = dr.seda_payload.clone();

            // save the data result
            let result_id = hash_data_result(&dr, block_height, exit_code, gas_used, &result);
            let dr_result = DataResult {
                version: dr.version,
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
        executor: Secpk256k1PublicKey,
    ) -> StdResult<GetCommittedDataResultResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let public_key_str = hex::encode(executor);
        let commitment = dr.commits.get(&public_key_str);
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
        executor: Secpk256k1PublicKey,
    ) -> StdResult<GetRevealedDataResultResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let public_key_str = hex::encode(executor);
        let reveal = dr.reveals.get(&public_key_str);
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
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        Ok(GetCommittedExecutorsResponse {
            value: dr.commits.keys().map(|k| k.clone().into_bytes()).collect(),
        })
    }

    /// Computes hash given a reveal and salt
    fn compute_hash(reveal: RevealBody) -> Hash {
        // hash non-fixed-length inputs
        let mut reveal_hasher = Keccak256::new();
        reveal_hasher.update(&reveal.reveal);
        let reveal_hash = reveal_hasher.finalize();

        // hash reveal body
        let mut reveal_body_hasher = Keccak256::new();
        reveal_body_hasher.update(reveal.salt);
        reveal_body_hasher.update(reveal.exit_code.to_be_bytes());
        reveal_body_hasher.update(reveal.gas_used.to_be_bytes());
        reveal_body_hasher.update(reveal_hash);
        reveal_body_hasher.finalize().into()
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
            public_key: vec![],
        };
        let info = mock_info("anyone", &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
