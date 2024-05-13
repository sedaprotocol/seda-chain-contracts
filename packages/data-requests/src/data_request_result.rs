use common::{
    msg::{GetCommittedDataResultResponse, GetRevealedDataResultResponse},
    types::Hash,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

pub mod data_request_results {

    use common::{
        crypto::{hash, recover_pubkey},
        error::ContractError::{
            self,
            AlreadyCommitted,
            AlreadyRevealed,
            IneligibleExecutor,
            NotCommitted,
            RevealMismatch,
            RevealNotStarted,
            RevealStarted,
        },
        msg::{
            GetCommittedDataResultsResponse,
            GetCommittedExecutorsResponse,
            GetResolvedDataResultResponse,
            GetRevealedDataResultsResponse,
        },
        state::{DataResult, RevealBody},
        types::{Secpk256k1PublicKey, Signature},
    };
    use cosmwasm_std::{Env, Event};
    use sha3::{Digest, Keccak256};

    use super::*;
    use crate::{
        contract::CONTRACT_VERSION,
        state::{DATA_REQUESTS_POOL, DATA_RESULTS},
        utils::{check_eligibility, hash_data_result, hash_to_string, validate_sender},
    };

    /// Posts a data result of a data request with an attached hash of the answer and salt.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn commit_result(
        deps: DepsMut,
        info: MessageInfo,
        dr_id: Hash,
        commitment: Hash,
        sender: Option<String>,
        signature: Signature,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // compute message hash
        let message_hash = hash(["commit_data_result".as_bytes(), &dr_id, &commitment, sender.as_bytes()]);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(message_hash, signature)?;
        let public_key_str = hex::encode(&public_key);
        if !check_eligibility(&deps, public_key)? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
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

        Ok(Response::new().add_attribute("action", "commit_data_result").add_event(
            Event::new("seda-commitment").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(&dr_id)),
                ("executor", sender.as_str()),
                ("commitment", &hash_to_string(&commitment)),
            ]),
        ))
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
        let reveal_body_hash = compute_hash(&reveal_body);

        // recover public key from signature
        let public_key: Secpk256k1PublicKey = recover_pubkey(reveal_body_hash, signature)?;
        let public_key_str = hex::encode(&public_key);
        if !check_eligibility(&deps, public_key)? {
            return Err(IneligibleExecutor);
        }

        // find the data request from the committed pool (if it exists, otherwise error)
        let mut dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;

        // error if reveal phase for this DR has not started (i.e. replication factor is not met)
        let committed_dr_results = &dr.commits;
        if u16::try_from(committed_dr_results.len()).unwrap() < dr.replication_factor {
            return Err(RevealNotStarted);
        }

        // error if data request executor has not submitted a commitment
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

        let mut response = Response::new().add_attribute("action", "reveal_data_result").add_event(
            Event::new("seda-reveal").add_attributes([
                ("version", CONTRACT_VERSION),
                ("dr_id", &hash_to_string(&dr_id)),
                ("executor", sender.as_str()),
                ("reveal", serde_json::to_string(&reveal_body).unwrap().as_str()),
            ]),
        );

        // add the reveal to the data request state
        let gas_used = reveal_body.gas_used;
        let reveal = reveal_body.reveal.clone();
        dr.reveals.insert(public_key_str, reveal_body);
        DATA_REQUESTS_POOL.update(deps.storage, dr_id, &dr)?;

        // if total reveals equals replication factor, resolve the DR
        // TODO: this needs to be separated out in a tally function once the module is implemented
        if u16::try_from(dr.reveals.len()).unwrap() == dr.replication_factor {
            let block_height: u64 = env.block.height;
            let exit_code: u8 = 0; // TODO: get this from the tally module

            // save the data result
            let result_id = hash_data_result(&dr, block_height, exit_code, gas_used, &reveal);

            // remove from the pool
            DATA_REQUESTS_POOL.remove(deps.storage, dr_id)?;

            response = response.add_event(Event::new("seda-data-result").add_attributes([
                ("version", CONTRACT_VERSION),
                ("result_id", &hash_to_string(&result_id)),
                ("dr_id", &hash_to_string(&dr_id)),
                ("block_height", &block_height.to_string()),
                ("exit_code", &exit_code.to_string()),
                ("result", &serde_json::to_string(&reveal).unwrap()),
                ("payback_address", &serde_json::to_string(&dr.payback_address).unwrap()),
                ("seda_payload", &serde_json::to_string(&dr.seda_payload).unwrap()),
            ]));

            let dr_result = DataResult {
                version: dr.version,
                dr_id,
                block_height,
                exit_code,
                result: reveal,
                payback_address: dr.payback_address,
                seda_payload: dr.seda_payload,
            };

            // save the data result
            DATA_RESULTS.save(deps.storage, dr_id, &dr_result)?;
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
        let commitment = dr.commits.get(&public_key_str).cloned();
        Ok(GetCommittedDataResultResponse { value: commitment })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_committed_data_results(deps: Deps, dr_id: Hash) -> StdResult<GetCommittedDataResultsResponse> {
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
        Ok(GetRevealedDataResultResponse { value: reveal.cloned() })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_revealed_data_results(deps: Deps, dr_id: Hash) -> StdResult<GetRevealedDataResultsResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        Ok(GetRevealedDataResultsResponse { value: dr.reveals })
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_resolved_data_result(deps: Deps, dr_id: Hash) -> StdResult<GetResolvedDataResultResponse> {
        let result = DATA_RESULTS.load(deps.storage, dr_id)?;
        Ok(GetResolvedDataResultResponse { value: result })
    }

    /// Returns a vector of committed executors
    pub fn get_committed_executors(deps: Deps, dr_id: Hash) -> StdResult<GetCommittedExecutorsResponse> {
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        Ok(GetCommittedExecutorsResponse {
            value: dr.commits.keys().cloned().map(|k| k.into_bytes()).collect(),
        })
    }

    /// Computes hash given a reveal and salt
    fn compute_hash(reveal: &RevealBody) -> Hash {
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
