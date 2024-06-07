use semver::{BuildMetadata, Prerelease, Version};
use sha3::{Digest, Keccak256};

use super::{
    msgs::data_requests::{execute, query},
    *,
};
use crate::{TestExecutor, TestInfo};

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> PostDataRequestArgs {
    let dr_binary_id: Hash = "dr_binary_id".hash();
    let tally_binary_id: Hash = "tally_binary_id".hash();
    let dr_inputs: Bytes = "dr_inputs".as_bytes().to_vec();
    let tally_inputs: Bytes = "tally_inputs".as_bytes().to_vec();

    // set by dr creator
    let gas_price = 10u128.into();
    let gas_limit = 10u128.into();

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

    PostDataRequestArgs {
        version,
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo,
        replication_factor,
        gas_price,
        gas_limit,
    }
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

impl TestInfo {
    #[track_caller]
    pub fn get_dr(&self, dr_id: Hash) -> Option<DataRequest> {
        self.query(query::QueryMsg::GetDataRequest { dr_id }).unwrap()
    }

    #[track_caller]
    pub fn post_data_request(
        &mut self,
        sender: &TestExecutor,
        posted_dr: PostDataRequestArgs,
        seda_payload: Vec<u8>,
        payback_address: Vec<u8>,
    ) -> Result<Hash, ContractError> {
        let msg = execute::post_request::Execute {
            posted_dr,
            seda_payload,
            payback_address,
        }
        .into();

        // someone posts a data request
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn commit_result(
        &mut self,
        sender: &TestExecutor,
        dr_id: Hash,
        commitment: Hash,
        msg_height: Option<u64>,
        env_height: Option<u64>,
    ) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "commit_data_result".as_bytes(),
            &dr_id,
            &msg_height.unwrap_or_default().to_be_bytes(),
            &commitment,
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::commit_result::Execute {
            dr_id,
            commitment,
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
        }
        .into();

        self.set_block_height(env_height.unwrap_or_default());
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn reveal_result(
        &mut self,
        sender: &TestExecutor,
        dr_id: Hash,
        reveal_body: RevealBody,
        msg_height: Option<u64>,
        env_height: Option<u64>,
    ) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "reveal_data_result".as_bytes(),
            &dr_id,
            &msg_height.unwrap_or_default().to_be_bytes(),
            &reveal_body.hash(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::reveal_result::Execute {
            reveal_body,
            dr_id,
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
        }
        .into();

        self.set_block_height(env_height.unwrap_or_default());
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn get_data_request(&self, dr_id: Hash) -> Option<DataRequest> {
        self.query(query::QueryMsg::GetDataRequest { dr_id }).unwrap()
    }

    #[track_caller]
    pub fn get_data_result_commit(&self, dr_id: Hash, public_key: Vec<u8>) -> Option<Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitment { dr_id, public_key })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_result_commits(&self, dr_id: Hash) -> HashMap<String, Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id })
            .unwrap()
    }

    pub fn get_data_result_reveal(&self, dr_id: Hash, public_key: Vec<u8>) -> Option<RevealBody> {
        self.query(query::QueryMsg::GetDataRequestReveal { dr_id, public_key })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_result_reveals(&self, dr_id: Hash) -> HashMap<String, RevealBody> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_requests_by_status(&self, status: DataRequestStatus) -> HashMap<String, DR> {
        self.query(query::QueryMsg::GetDataRequestsByStatus { status }).unwrap()
    }
}
