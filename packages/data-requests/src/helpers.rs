use std::collections::HashMap;

use crate::state::DataRequestInputs;
use crate::utils::{hash_data_request, string_to_hash};
use common::msg::PostDataRequestArgs;
use common::state::{DataRequest, Reveal};
use common::types::Hash;
use common::types::{Bytes, Commitment};

use semver::{BuildMetadata, Prerelease, Version};
use sha3::Digest;
use sha3::Keccak256;

use cosmwasm_std::testing::mock_env;

use crate::contract::{instantiate, query};
use common::msg::{GetDataRequestsFromPoolResponse, InstantiateMsg};
use common::{error::ContractError, msg::GetDataRequestResponse};
use cosmwasm_std::from_binary;

use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn calculate_dr_id_and_args(
    nonce: u128,
    replication_factor: u16,
) -> (Hash, PostDataRequestArgs) {
    let dr_binary_id: Hash = string_to_hash("dr_binary_id");
    let tally_binary_id: Hash = string_to_hash("tally_binary_id");
    let dr_inputs: Bytes = "dr_inputs".as_bytes().to_vec();
    let tally_inputs: Bytes = "tally_inputs".as_bytes().to_vec();

    // set by dr creator
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;

    // set by relayer and SEDA protocol
    let seda_payload: Bytes = Vec::new();
    let payback_address: Bytes = Vec::new();

    // memo
    let chain_id: u128 = 31337;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo = hasher.finalize().to_vec();

    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let constructed_dr_input = DataRequestInputs {
        version: version.clone(),
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
    let constructed_dr_id = hash_data_request(constructed_dr_input);

    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        version,
        dr_id: constructed_dr_id.clone(),
        dr_binary_id,
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

    (constructed_dr_id, posted_dr)
}

pub fn construct_dr(constructed_dr_id: Hash, dr_args: PostDataRequestArgs) -> DataRequest {
    let commits: HashMap<String, Commitment> = HashMap::new();
    let reveals: HashMap<String, Reveal> = HashMap::new();

    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let payback_address: Bytes = Vec::new();
    DataRequest {
        version,
        dr_id: constructed_dr_id,

        dr_binary_id: dr_args.dr_binary_id,
        tally_binary_id: dr_args.tally_binary_id,
        dr_inputs: dr_args.dr_inputs,
        tally_inputs: dr_args.tally_inputs,
        memo: dr_args.memo,
        replication_factor: dr_args.replication_factor,
        gas_price: dr_args.gas_price,
        gas_limit: dr_args.gas_limit,
        seda_payload: dr_args.seda_payload,
        commits,
        reveals,
        payback_address,
    }
}

pub fn instantiate_dr_contract(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        proxy: "proxy".to_string(),
    };
    instantiate(deps, mock_env(), info, msg)
}

pub fn get_dr(deps: DepsMut, dr_id: Hash) -> GetDataRequestResponse {
    let res = query(
        deps.as_ref(),
        mock_env(),
        common::msg::DataRequestsQueryMsg::GetDataRequest { dr_id },
    )
    .unwrap();
    let value: GetDataRequestResponse = from_binary(&res).unwrap();
    value
}

pub fn get_drs_from_pool(
    deps: DepsMut,
    position: Option<u128>,
    limit: Option<u128>,
) -> GetDataRequestsFromPoolResponse {
    let res = query(
        deps.as_ref(),
        mock_env(),
        common::msg::DataRequestsQueryMsg::GetDataRequestsFromPool { position, limit },
    )
    .unwrap();
    let value: GetDataRequestsFromPoolResponse = from_binary(&res).unwrap();
    value
}
