use common::error::ContractError;
use common::msg::StakingQueryMsg;
use common::msg::{IsDataRequestExecutorEligibleResponse, PostDataRequestArgs};
use common::state::DataRequest;
use common::types::{Bytes, Hash, Secpk256k1PublicKey};
use cosmwasm_std::{to_json_binary, Addr, DepsMut, QueryRequest, WasmQuery};
use sha3::{Digest, Keccak256};

use crate::state::PROXY_CONTRACT;

pub fn check_eligibility(
    deps: &DepsMut,
    dr_executor: Secpk256k1PublicKey,
) -> Result<bool, ContractError> {
    // query proxy contract to see if this executor is eligible
    let msg = StakingQueryMsg::IsDataRequestExecutorEligible {
        executor: dr_executor,
    };
    let query_response: IsDataRequestExecutorEligibleResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: PROXY_CONTRACT.load(deps.storage)?.to_string(),
            msg: to_json_binary(&msg)?,
        }))?;
    Ok(query_response.value)
}

pub fn hash_data_request(posted_dr: &PostDataRequestArgs) -> Hash {
    // hash non-fixed-length inputs
    let mut dr_inputs_hasher = Keccak256::new();
    dr_inputs_hasher.update(&posted_dr.dr_inputs);
    let dr_inputs_hash = dr_inputs_hasher.finalize();

    let mut tally_inputs_hasher = Keccak256::new();
    tally_inputs_hasher.update(&posted_dr.tally_inputs);
    let tally_inputs_hash = tally_inputs_hasher.finalize();

    let mut memo_hasher = Keccak256::new();
    memo_hasher.update(&posted_dr.memo);
    let memo_hash = memo_hasher.finalize();

    // hash data request
    let mut dr_hasher = Keccak256::new();
    dr_hasher.update(posted_dr.version.to_string().as_bytes());
    dr_hasher.update(posted_dr.dr_binary_id);
    dr_hasher.update(dr_inputs_hash);
    dr_hasher.update(posted_dr.tally_binary_id);
    dr_hasher.update(tally_inputs_hash);
    dr_hasher.update(posted_dr.replication_factor.to_be_bytes());
    dr_hasher.update(posted_dr.gas_price.to_be_bytes());
    dr_hasher.update(posted_dr.gas_limit.to_be_bytes());
    dr_hasher.update(memo_hash);
    dr_hasher.finalize().into()
}

pub fn hash_data_result(
    dr: &DataRequest,
    block_height: u64,
    exit_code: u8,
    gas_used: u128,
    result: &Bytes,
) -> Hash {
    // hash non-fixed-length inputs
    let mut results_hasher = Keccak256::new();
    results_hasher.update(result); // TODO check this
    let results_hash = results_hasher.finalize();

    let mut payback_address_hasher = Keccak256::new();
    payback_address_hasher.update(&dr.payback_address);
    let payback_address_hash = payback_address_hasher.finalize();

    let mut seda_payload_hasher = Keccak256::new();
    seda_payload_hasher.update(&dr.seda_payload);
    let seda_payload_hash = seda_payload_hasher.finalize();

    // hash data result
    let mut dr_hasher = Keccak256::new();
    dr_hasher.update(dr.id);
    dr_hasher.update(block_height.to_be_bytes());
    dr_hasher.update(exit_code.to_be_bytes());
    dr_hasher.update(results_hash);
    dr_hasher.update(gas_used.to_be_bytes());
    dr_hasher.update(payback_address_hash);
    dr_hasher.update(seda_payload_hash);
    dr_hasher.finalize().into()
}

pub fn validate_sender(
    deps: &DepsMut,
    caller: Addr,
    sender: Option<String>,
) -> Result<Addr, ContractError> {
    // if a sender is passed, caller must be the proxy contract
    match sender {
        Some(_sender) if caller != PROXY_CONTRACT.load(deps.storage)? => {
            Err(ContractError::NotProxy {})
        }
        Some(sender) => Ok(deps.api.addr_validate(&sender)?),
        None => Ok(caller),
    }
}

pub fn string_to_hash(input: &str) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

pub fn hash_to_string(input: Hash) -> String {
    hex::encode(input)
}
