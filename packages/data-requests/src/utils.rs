use crate::state::{DataRequestInputs, PROXY_CONTRACT};
use common::error::ContractError;
use common::msg::{IsDataRequestExecutorEligibleResponse, StakingQueryMsg};
use common::state::DataRequest;
use common::types::{Bytes, Hash};
use cosmwasm_std::{to_json_binary, Addr, DepsMut, QueryRequest, WasmQuery};
use sha3::{Digest, Keccak256};

pub fn check_eligibility(deps: &DepsMut, dr_executor: Addr) -> Result<bool, ContractError> {
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

pub fn hash_data_request(posted_dr: DataRequestInputs) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(posted_dr.dr_binary_id);
    hasher.update(posted_dr.dr_inputs);
    hasher.update(posted_dr.gas_limit.to_be_bytes());
    hasher.update(posted_dr.gas_price.to_be_bytes());
    hasher.update(posted_dr.memo);
    hasher.update(posted_dr.replication_factor.to_be_bytes());
    hasher.update(posted_dr.tally_binary_id);
    hasher.update(posted_dr.tally_inputs);
    hasher.finalize().into()
}

pub fn hash_seed(seed: &str, dr_id: &Hash) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(seed);
    hasher.update(dr_id);
    hasher.finalize().into()

    // format!("0x{}", hex::encode(hasher.finalize()))
}

pub fn hash_data_result(
    dr: &DataRequest,
    block_height: u64,
    exit_code: u8,
    result: &Bytes,
) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(dr.dr_id);
    hasher.update(block_height.to_be_bytes());
    hasher.update(exit_code.to_be_bytes());
    hasher.update(result);
    hasher.update(dr.payback_address.clone());
    hasher.update(dr.seda_payload.clone());
    hasher.finalize().into()
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
