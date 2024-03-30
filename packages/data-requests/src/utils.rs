use alloy_sol_types::SolType;
use common::error::ContractError;
use common::msg::{IsDataRequestExecutorEligibleResponse, StakingQueryMsg};
use common::state::DataRequest;
use common::types::{Bytes, Hash};
use cosmwasm_std::{to_json_binary, Addr, DepsMut, QueryRequest, WasmQuery};
use sha3::{Digest, Keccak256};

use crate::state::{DataRequestInputs, PROXY_CONTRACT};
use crate::types::{DataRequestHashInputs, DataResultHashInputs};

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
    let data_requests_hash_inputs = DataRequestHashInputs {
        version: posted_dr.version.to_string(),
        dr_binary_id: alloy_sol_types::private::FixedBytes(posted_dr.dr_binary_id),
        dr_inputs: posted_dr.dr_inputs,
        gas_limit: posted_dr.gas_limit,
        gas_price: posted_dr.gas_price,
        tally_gas_limit: posted_dr.tally_gas_limit,
        memo: posted_dr.memo,
        replication_factor: posted_dr.replication_factor,
        tally_binary_id: alloy_sol_types::private::FixedBytes(posted_dr.tally_binary_id),
        tally_inputs: posted_dr.tally_inputs,
    };
    let mut hasher = Keccak256::new();
    hasher.update(DataRequestHashInputs::abi_encode_params(
        &data_requests_hash_inputs,
    ));
    hasher.finalize().into()
}

pub fn hash_data_result(
    dr: &DataRequest,
    block_height: u64,
    exit_code: u8,
    result: &Bytes,
) -> Hash {
    let data_results_hash_inputs = DataResultHashInputs {
        version: dr.version.to_string(),
        dr_id: alloy_sol_types::private::FixedBytes(dr.dr_id),
        block_height: block_height.into(),
        exit_code,
        result: result.clone(),
        payback_address: dr.payback_address.clone(),
        seda_payload: dr.seda_payload.clone(),
    };

    let mut hasher = Keccak256::new();
    hasher.update(DataResultHashInputs::abi_encode_params(
        &data_results_hash_inputs,
    ));
    hasher.finalize().into()
}

pub fn caller_is_proxy(deps: &DepsMut, caller: Addr) -> Result<(), ContractError> {
    if caller != PROXY_CONTRACT.load(deps.storage)? {
        Err(ContractError::NotProxy {})
    } else {
        Ok(())
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
