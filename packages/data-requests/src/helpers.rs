use std::collections::HashMap;

use crate::data_request_result::data_request_results::get_seed;
use crate::state::DataRequestInputs;
use crate::utils::{hash_data_request, hash_seed, string_to_hash};
use common::msg::PostDataRequestArgs;
use common::state::{DataRequest, Reveal};
use common::types::Hash;
use common::types::{Bytes, Commitment};

use sha3::Digest;
use sha3::Keccak256;

use cosmwasm_std::testing::mock_env;

use crate::contract::{instantiate, query};
use common::msg::{
    GetDataRequestsFromPoolResponse, InstantiateMsg, QuerySeedResponse, SpecialQueryWrapper,
};
use common::{error::ContractError, msg::GetDataRequestResponse};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, Deps, DepsMut, MessageInfo, OwnedDeps, Querier, QuerierResult,
    QueryRequest, Response, SystemError, SystemResult,
};

pub fn calculate_dr_id_and_args(
    nonce: u128,
    replication_factor: u16,
) -> (Hash, PostDataRequestArgs) {
    let dr_binary_id: Hash = string_to_hash("dr_binary_id");
    let tally_binary_id: Hash = string_to_hash("tally_binary_id");
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();

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

    let constructed_dr_input = DataRequestInputs {
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

pub fn construct_dr(
    deps: Deps<SpecialQueryWrapper>,
    constructed_dr_id: Hash,
    dr_args: PostDataRequestArgs,
) -> DataRequest {
    let commits: HashMap<String, Commitment> = HashMap::new();
    let reveals: HashMap<String, Reveal> = HashMap::new();
    let payback_address: Bytes = Vec::new();
    let seed_hash = hash_seed(
        get_seed(deps.into_empty()).unwrap().seed,
        constructed_dr_id.clone(),
    );
    DataRequest {
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
        seed_hash,
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

pub fn get_dr(deps: DepsMut<SpecialQueryWrapper>, dr_id: Hash) -> GetDataRequestResponse {
    let res = query(
        deps.into_empty().as_ref(),
        mock_env(),
        common::msg::DataRequestsQueryMsg::GetDataRequest { dr_id },
    )
    .unwrap();
    let value: GetDataRequestResponse = from_json(&res).unwrap();
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
    let value: GetDataRequestsFromPoolResponse = from_json(&res).unwrap();
    value
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, SpecialQueryWrapper>
{
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
        custom_query_type: std::marker::PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<SpecialQueryWrapper>,
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<SpecialQueryWrapper>) -> Self {
        WasmMockQuerier { base }
    }

    pub fn handle_query(&self, request: &QueryRequest<SpecialQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(SpecialQueryWrapper { query_data: _ }) => {
                let res = QuerySeedResponse {
                    seed: "seed".to_string(),
                    block_height: 1,
                };
                SystemResult::Ok(to_json_binary(&res).into())
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<SpecialQueryWrapper> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}
