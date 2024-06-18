use std::collections::HashMap;

use semver::{BuildMetadata, Prerelease, Version};
use sha3::{Digest, Keccak256};

use super::{
    msgs::data_requests::{execute, query, sudo},
    *,
};
use crate::{TestExecutor, TestInfo};

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> PostDataRequestArgs {
    let dr_binary_id = nonce.to_string().hash().to_hex();
    let tally_binary_id = "tally_binary_id".hash().to_hex();
    let dr_inputs = "dr_inputs".as_bytes().into();
    let tally_inputs = "tally_inputs".as_bytes().into();

    // set by dr creator
    let gas_price = 10u128.into();
    let gas_limit = 10u128.into();

    // memo
    let chain_id: u128 = 31337;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo = hasher.finalize();

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
        memo: memo.as_slice().into(),
        replication_factor,
        gas_price,
        gas_limit,
    }
}

pub fn construct_dr(
    constructed_dr_id: Hash,
    dr_args: PostDataRequestArgs,
    seda_payload: Vec<u8>,
    height: u64,
) -> DataRequest {
    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre:   Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let payback_address: Vec<u8> = Vec::new();
    DataRequest {
        version,
        id: constructed_dr_id.to_hex(),

        dr_binary_id: dr_args.dr_binary_id,
        tally_binary_id: dr_args.tally_binary_id,
        dr_inputs: dr_args.dr_inputs,
        tally_inputs: dr_args.tally_inputs,
        memo: dr_args.memo,
        replication_factor: dr_args.replication_factor,
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
    pub fn get_dr(&self, dr_id: Hash) -> Option<DataRequest> {
        self.query(query::QueryMsg::GetDataRequest { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    #[track_caller]
    pub fn post_data_request(
        &mut self,
        sender: &TestExecutor,
        posted_dr: PostDataRequestArgs,
        seda_payload: Vec<u8>,
        payback_address: Vec<u8>,
        env_height: u64,
    ) -> Result<Hash, ContractError> {
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
    pub fn commit_result(&mut self, sender: &TestExecutor, dr_id: Hash, commitment: Hash) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let dr = self.get_dr(dr_id).unwrap();
        let dr_id = dr_id.to_hex();
        let commitment = commitment.to_hex();
        let msg_hash = hash([
            "commit_data_result".as_bytes(),
            dr_id.as_bytes(),
            &dr.height.to_be_bytes(),
            commitment.as_bytes(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::commit_result::Execute {
            dr_id,
            commitment,
            public_key: sender.pub_key_hex(),
            proof: sender.prove_hex(&msg_hash),
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn commit_result_wrong_height(
        &mut self,
        sender: &TestExecutor,
        dr_id: Hash,
        commitment: Hash,
    ) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let dr = self.get_dr(dr_id).unwrap();
        let dr_id = dr_id.to_hex();
        let commitment = commitment.to_hex();
        let msg_hash = hash([
            "commit_data_result".as_bytes(),
            dr_id.as_bytes(),
            &dr.height.saturating_sub(3).to_be_bytes(),
            commitment.as_bytes(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::commit_result::Execute {
            dr_id,
            commitment,
            public_key: sender.pub_key_hex(),
            proof: sender.prove_hex(&msg_hash),
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn reveal_result(
        &mut self,
        sender: &TestExecutor,
        dr_id: Hash,
        reveal_body: RevealBody,
    ) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let dr = self.get_dr(dr_id).unwrap();
        let dr_id = dr_id.to_hex();
        let msg_hash = hash([
            "reveal_data_result".as_bytes(),
            dr_id.as_bytes(),
            &dr.height.to_be_bytes(),
            &reveal_body.hash(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::reveal_result::Execute {
            reveal_body,
            dr_id,
            public_key: sender.pub_key_hex(),
            proof: sender.prove_hex(&msg_hash),
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn post_data_result(&mut self, dr_id: Hash, result: DataResult, exit_code: u8) -> Result<(), ContractError> {
        let msg = sudo::post_result::Sudo {
            dr_id: dr_id.to_hex(),
            result,
            exit_code,
        }
        .into();
        self.sudo(&msg)
    }

    #[track_caller]
    pub fn get_data_request(&self, dr_id: Hash) -> DataRequest {
        self.query(query::QueryMsg::GetDataRequest { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_result(&self, dr_id: Hash) -> DataResult {
        dbg!(self.query(query::QueryMsg::GetDataResult { dr_id: dr_id.to_hex() })).unwrap()
    }

    #[track_caller]
    pub fn get_data_result_commit(&self, dr_id: Hash, public_key: PublicKey) -> Option<Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitment {
            dr_id:      dr_id.to_hex(),
            public_key: public_key.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_result_commits(&self, dr_id: Hash) -> HashMap<String, Hash> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    pub fn get_data_result_reveal(&self, dr_id: Hash, public_key: PublicKey) -> Option<RevealBody> {
        self.query(query::QueryMsg::GetDataRequestReveal {
            dr_id:      dr_id.to_hex(),
            public_key: public_key.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_data_result_reveals(&self, dr_id: Hash) -> HashMap<String, RevealBody> {
        self.query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_requests_by_status(&self, status: DataRequestStatus, offset: u32, limit: u32) -> Vec<DataRequest> {
        self.query(query::QueryMsg::GetDataRequestsByStatus { status, offset, limit })
            .unwrap()
    }
}
