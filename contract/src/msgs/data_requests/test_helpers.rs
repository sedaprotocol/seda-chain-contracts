use std::collections::HashMap;

use msgs::data_requests::sudo::expire_data_requests;
use semver::{BuildMetadata, Prerelease, Version};
use sha3::{Digest, Keccak256};

use super::{
    msgs::data_requests::{execute, query, sudo},
    *,
};
use crate::{TestExecutor, TestInfo};

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> PostDataRequestArgs {
    let exec_program_id = nonce.to_string().hash().to_hex();
    let tally_program_id = "tally_program_id".hash().to_hex();
    let dr_inputs = "dr_inputs".as_bytes().into();
    let tally_inputs = "tally_inputs".as_bytes().into();

    // set by dr creator
    let gas_price = 10;
    let gas_limit = 10;

    // memo
    let chain_id: u128 = 31337;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo = hasher.finalize();

    let version = Version {
        major: 0,
        minor: 0,
        patch: 1,
        pre:   Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let consensus_filter = vec![0u8].into();

    PostDataRequestArgs {
        version,
        exec_program_id,
        tally_program_id,
        dr_inputs,
        tally_inputs,
        memo: memo.as_slice().into(),
        replication_factor,
        consensus_filter,
        gas_price,
        gas_limit,
    }
}

pub fn construct_dr(dr_args: PostDataRequestArgs, seda_payload: Vec<u8>, height: u64) -> DataRequest {
    let version = Version {
        major: 0,
        minor: 0,
        patch: 1,
        pre:   Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };
    let dr_id = dr_args.try_hash().unwrap();

    let payback_address: Vec<u8> = vec![1, 2, 3];
    DataRequest {
        version,
        id: dr_id.to_hex(),
        exec_program_id: dr_args.exec_program_id,
        tally_program_id: dr_args.tally_program_id,
        dr_inputs: dr_args.dr_inputs,
        tally_inputs: dr_args.tally_inputs,
        memo: dr_args.memo,
        replication_factor: dr_args.replication_factor,
        consensus_filter: dr_args.consensus_filter,
        gas_price: dr_args.gas_price,
        gas_limit: dr_args.gas_limit,
        seda_payload: seda_payload.into(),
        commits: Default::default(),
        reveals: Default::default(),
        payback_address: payback_address.into(),

        height,
    }
}

pub fn construct_result(dr: DataRequest, reveal: RevealBody, exit_code: u8) -> DataResult {
    DataResult {
        version: dr.version,
        dr_id: dr.id,
        block_height: dr.height,
        exit_code,
        gas_used: reveal.gas_used,
        result: reveal.reveal,
        payback_address: dr.payback_address,
        seda_payload: dr.seda_payload,
        consensus: true,
    }
}

impl TestInfo {
    #[track_caller]
    pub fn post_data_request(
        &mut self,
        sender: &TestExecutor,
        posted_dr: PostDataRequestArgs,
        seda_payload: Vec<u8>,
        payback_address: Vec<u8>,
        env_height: u64,
    ) -> Result<String, ContractError> {
        let msg = execute::post_request::Execute {
            posted_dr,
            seda_payload: seda_payload.into(),
            payback_address: payback_address.into(),
        }
        .into();

        if env_height < self.block_height() {
            panic!("Invalid Test: Cannot post a data request in the past");
        }
        // set the chain height... will effect the height in the dr for us to sign.
        self.set_block_height(env_height);
        // someone posts a data request
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn commit_result(&mut self, sender: &TestExecutor, dr_id: &str, commitment: Hash) -> Result<(), ContractError> {
        let dr = self.get_data_request(dr_id).unwrap();
        let commitment = commitment.to_hex();

        let factory = execute::commit_result::Execute::factory(
            dr_id.to_string(),
            commitment,
            sender.pub_key_hex(),
            self.chain_id(),
            self.contract_addr(),
            dr.height,
        );
        let proof = sender.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn commit_result_wrong_height(
        &mut self,
        sender: &TestExecutor,
        dr_id: &str,
        commitment: Hash,
    ) -> Result<(), ContractError> {
        let dr = self.get_data_request(dr_id).unwrap();
        let commitment = commitment.to_hex();

        let factory = execute::commit_result::Execute::factory(
            dr_id.to_string(),
            commitment,
            sender.pub_key_hex(),
            self.chain_id(),
            self.contract_addr(),
            dr.height.saturating_sub(3),
        );
        let proof = sender.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn reveal_result(
        &mut self,
        sender: &TestExecutor,
        dr_id: &str,
        reveal_body: RevealBody,
    ) -> Result<(), ContractError> {
        let dr = self.get_data_request(dr_id).unwrap();
        let reveal_body_hash = reveal_body.try_hash()?;

        let factory = execute::reveal_result::Execute::factory(
            dr_id.to_string(),
            reveal_body,
            sender.pub_key_hex(),
            vec![],
            vec![],
            self.chain_id(),
            self.contract_addr(),
            dr.height,
            reveal_body_hash,
        );
        let proof = sender.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn post_data_result(&mut self, dr_id: String, result: DataResult, exit_code: u8) -> Result<(), ContractError> {
        let msg = sudo::PostResult {
            dr_id,
            result,
            exit_code,
        }
        .into();
        self.sudo(&msg)
    }

    #[track_caller]
    pub fn post_data_results(&mut self, results: Vec<(String, DataResult, u8)>) -> Result<(), ContractError> {
        let msg = sudo::post_results::Sudo {
            results: results
                .into_iter()
                .map(|(dr_id, result, exit_code)| sudo::PostResult {
                    dr_id,
                    result,
                    exit_code,
                })
                .collect(),
        }
        .into();
        self.sudo(&msg)
    }

    #[track_caller]
    pub fn get_data_request(&self, dr_id: &str) -> Option<DataRequest> {
        self.query(query::QueryMsg::GetDataRequest {
            dr_id: dr_id.to_string(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_result(&self, dr_id: &str) -> Option<DataResult> {
        self.query(query::QueryMsg::GetDataResult {
            dr_id: dr_id.to_string(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_commit(&self, dr_id: Hash, public_key: PublicKey) -> Option<Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitment {
            dr_id:      dr_id.to_hex(),
            public_key: public_key.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_commits(&self, dr_id: Hash) -> HashMap<String, Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    pub fn get_data_request_reveal(&self, dr_id: Hash, public_key: PublicKey) -> Option<RevealBody> {
        self.query(query::QueryMsg::GetDataRequestReveal {
            dr_id:      dr_id.to_hex(),
            public_key: public_key.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_reveals(&self, dr_id: Hash) -> HashMap<String, RevealBody> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_requests_by_status(&self, status: DataRequestStatus, offset: u32, limit: u32) -> Vec<DataRequest> {
        self.query(query::QueryMsg::GetDataRequestsByStatus { status, offset, limit })
            .unwrap()
    }

    #[track_caller]
    pub fn expire_data_requests(&mut self) -> Result<(), ContractError> {
        let msg = expire_data_requests::Sudo {}.into();
        self.sudo(&msg)
    }

    #[track_caller]
    pub fn set_timeout_config(
        &mut self,
        sender: &TestExecutor,
        timeout_config: TimeoutConfig,
    ) -> Result<(), ContractError> {
        let msg = execute::ExecuteMsg::SetTimeoutConfig(timeout_config).into();
        self.execute(sender, &msg)
    }
}
