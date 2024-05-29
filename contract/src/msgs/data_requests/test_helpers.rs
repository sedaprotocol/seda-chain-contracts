use cosmwasm_std::{from_json, testing::mock_env};
use semver::{BuildMetadata, Prerelease};

use super::{execute::*, *};
use crate::{
    contract::{execute, query},
    crypto::hash,
    TestExecutor,
};

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> (Hash, PostDataRequestArgs) {
    let dr_binary_id: Hash = "dr_binary_id".hash();
    let tally_binary_id: Hash = "tally_binary_id".hash();
    let dr_inputs: Bytes = "dr_inputs".as_bytes().to_vec();
    let tally_inputs: Bytes = "tally_inputs".as_bytes().to_vec();

    // set by dr creator
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;

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
        pre:   Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        version,
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo,
        replication_factor,
        gas_price,
        gas_limit,
    };

    (posted_dr.hash(), posted_dr)
}

pub fn construct_dr(constructed_dr_id: Hash, dr_args: PostDataRequestArgs, seda_payload: Bytes) -> DataRequest {
    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre:   Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let payback_address: Bytes = Vec::new();
    DataRequest {
        version,
        id: constructed_dr_id,

        dr_binary_id: dr_args.dr_binary_id,
        tally_binary_id: dr_args.tally_binary_id,
        dr_inputs: dr_args.dr_inputs,
        tally_inputs: dr_args.tally_inputs,
        memo: dr_args.memo,
        replication_factor: dr_args.replication_factor,
        gas_price: dr_args.gas_price,
        gas_limit: dr_args.gas_limit,
        seda_payload,
        commits: Default::default(),
        reveals: Default::default(),
        payback_address,
    }
}

pub fn get_dr(deps: DepsMut, dr_id: Hash) -> Option<DataRequest> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequest { dr_id }.into(),
    )
    .unwrap();
    let value: Option<DataRequest> = from_json(res).unwrap();
    value
}

pub fn post_data_request(
    deps: DepsMut,
    info: MessageInfo,
    posted_dr: PostDataRequestArgs,
    seda_payload: Vec<u8>,
    payback_address: Vec<u8>,
) -> Result<Response, ContractError> {
    let msg = post_request::Execute {
        posted_dr,
        seda_payload,
        payback_address,
    };
    // someone posts a data request
    execute(deps, mock_env(), info.clone(), msg.into())
}

fn env_with_height(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}

pub fn commit_result(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    dr_id: Hash,
    commitment: Hash,
    msg_height: Option<u64>,
    env_height: Option<u64>,
) -> Result<Response, ContractError> {
    let msg_hash = hash([
        "commit_data_result".as_bytes(),
        &dr_id,
        &msg_height.unwrap_or_default().to_be_bytes(),
        &commitment,
    ]);

    let msg = commit_result::Execute {
        dr_id,
        commitment,
        public_key: exec.pub_key(),
        proof: exec.prove(&msg_hash),
    };
    execute(
        deps,
        env_with_height(env_height.unwrap_or_default()),
        info.clone(),
        msg.into(),
    )
}

pub fn reveal_result(
    deps: DepsMut,
    info: MessageInfo,
    exec: &TestExecutor,
    dr_id: Hash,
    reveal_body: RevealBody,
    msg_height: Option<u64>,
    env_height: Option<u64>,
) -> Result<Response, ContractError> {
    let msg_hash = hash([
        "reveal_data_result".as_bytes(),
        &dr_id,
        &msg_height.unwrap_or_default().to_be_bytes(),
        &reveal_body.hash(),
    ]);

    let msg = reveal_result::Execute {
        reveal_body,
        dr_id,
        public_key: exec.pub_key(),
        proof: exec.prove(&msg_hash),
    };

    execute(
        deps,
        env_with_height(env_height.unwrap_or_default()),
        info.clone(),
        msg.into(),
    )
}

pub fn get_data_request(deps: DepsMut, dr_id: Hash) -> Option<DataRequest> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequest { dr_id }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}

pub fn get_data_result_commit(deps: DepsMut, dr_id: Hash, public_key: Vec<u8>) -> Option<Hash> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequestCommitment { dr_id, public_key }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}

pub fn get_data_result_commits(deps: DepsMut, dr_id: Hash) -> HashMap<String, Hash> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequestCommitments { dr_id }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}

pub fn get_data_result_reveal(deps: DepsMut, dr_id: Hash, public_key: Vec<u8>) -> Option<RevealBody> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequestReveal { dr_id, public_key }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}

pub fn get_data_result_reveals(deps: DepsMut, dr_id: Hash) -> HashMap<String, RevealBody> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequestCommitments { dr_id }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}

pub fn get_data_requests_by_status(deps: DepsMut, status: DataRequestStatus) -> HashMap<String, DR> {
    let res = query(
        deps.as_ref(),
        mock_env(),
        query::QueryMsg::GetDataRequestsByStatus { status }.into(),
    )
    .unwrap();
    from_json(res).unwrap()
}
