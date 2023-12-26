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

pub fn encode_data_as_abi_bytes(input: &[u8]) -> Vec<u8> {
    // Corrected offset to the data
    let mut offset = [0u8; 32];
    offset[31] = 0x20; // Set the last byte to 0x20

    // Length of the data in bytes, as a 32-byte array
    let length = (input.len() as u64).to_be_bytes();
    let mut length_32 = [0u8; 32];
    length_32[24..32].copy_from_slice(&length);

    // Combine offset, length, and data
    let mut combined = Vec::new();
    combined.extend_from_slice(&offset);
    combined.extend_from_slice(&length_32);
    combined.extend_from_slice(input);

    // Pad the data to 32 bytes boundary if necessary
    let padding_needed = 32 - (input.len() % 32);
    if padding_needed < 32 {
        combined.extend(vec![0u8; padding_needed]);
    }

    combined
}

fn left_pad_to_32_bytes(data: &[u8]) -> Vec<u8> {
    let mut padded = vec![0; 32 - data.len()];
    padded.extend_from_slice(data);
    padded
}

pub fn concat_abi_encoded(version: &[u8], dr_binary_id: &[u8; 32]) -> Vec<u8> {
    let mut combined = Vec::new();

    // Offset for version (32 bytes after dr_binary_id)
    let offset = [0u8; 32];
    combined.extend_from_slice(&offset);
    combined[31] = 0x40; // Set the last byte to 0x40, indicating an offset of 64 bytes

    // Append dr_binary_id (static type)
    combined.extend_from_slice(dr_binary_id);

    // Length of the version in bytes, as a 32-byte array
    let length = (version.len() as u64).to_be_bytes();
    let mut length_32 = [0u8; 32];
    length_32[24..32].copy_from_slice(&length);

    // Append length and actual data for version
    combined.extend_from_slice(&length_32);
    combined.extend_from_slice(version);

    // Pad the version data to 32 bytes boundary if necessary
    let padding_needed = 32 - (version.len() % 32);
    if padding_needed < 32 {
        combined.extend(vec![0u8; padding_needed]);
    }

    combined
}

pub fn hash_data_request(posted_dr: DataRequestInputs) -> Hash {
    
    let binding = posted_dr.version.clone().to_string();
    let version_bytes = binding.as_bytes();
    let version_and_dr_binary_id = concat_abi_encoded(version_bytes, &posted_dr.dr_binary_id);
    println!("version_and_dr_binary_id: 0x{}", hex::encode(version_and_dr_binary_id.clone()));

    
    // // testing
    // println!("version: 0x{}", hex::encode(encode_data_as_abi_bytes(posted_dr.version.to_string().as_bytes())));
    // println!("dr_binary_id: 0x{}", hex::encode(posted_dr.dr_binary_id));

    // let mut version_and_dr_binary_id = encode_data_as_abi_bytes(posted_dr.version.to_string().as_bytes());
    // version_and_dr_binary_id.append(&mut posted_dr.dr_binary_id.to_vec());
    // println!("version_and_dr_binary_id: 0x{:?}", hex::encode(version_and_dr_binary_id));
    
    
    let mut hasher = Keccak256::new();

    hasher.update(version_and_dr_binary_id);

    // hasher.update(encode_data_as_abi_bytes(posted_dr.version.to_string().as_bytes()));
    // hasher.update(posted_dr.dr_binary_id);

    // hasher.update(encode_data_as_abi_bytes(&posted_dr.dr_inputs));
    // hasher.update(left_pad_to_32_bytes(&posted_dr.gas_limit.to_be_bytes()));
    // hasher.update(left_pad_to_32_bytes(&posted_dr.gas_price.to_be_bytes()));
    // hasher.update(left_pad_to_32_bytes(&posted_dr.tally_gas_limit.to_be_bytes()));
    // hasher.update(posted_dr.memo);
    // hasher.update(left_pad_to_32_bytes(&posted_dr.replication_factor.to_be_bytes()));
    // hasher.update(posted_dr.tally_binary_id);
    // hasher.update(encode_data_as_abi_bytes(&posted_dr.tally_inputs));

    hasher.finalize().into()
}

pub fn hash_seed(seed: String, dr_id: Hash) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(seed);
    hasher.update(dr_id);
    hasher.finalize().into()
}

pub fn hash_data_result(
    dr: &DataRequest,
    block_height: u64,
    exit_code: u8,
    result: &Bytes,
) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(dr.version.to_string());
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
