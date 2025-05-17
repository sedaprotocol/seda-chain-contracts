use std::collections::HashMap;

use msgs::data_requests::sudo::{expire_data_requests, DistributionMessage};
use seda_common::msgs::data_requests::execute::reveal_result::Execute as RevealMessage;
use semver::{BuildMetadata, Prerelease, Version};
use sha3::{Digest, Keccak256};

use super::{
    consts::{MIN_EXEC_GAS_LIMIT, MIN_GAS_PRICE, MIN_TALLY_GAS_LIMIT},
    msgs::data_requests::{execute, query, sudo},
    *,
};
use crate::{msgs::data_requests::consts::min_post_dr_cost, TestAccount};

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> PostDataRequestArgs {
    let exec_program_id = nonce.to_string().hash().to_hex();
    let tally_program_id = "tally_program_id".hash().to_hex();
    let exec_inputs = "exec_inputs".as_bytes().into();
    let tally_inputs = "tally_inputs".as_bytes().into();

    // set by dr creator
    let gas_price = MIN_GAS_PRICE;
    let exec_gas_limit = MIN_EXEC_GAS_LIMIT;
    let tally_gas_limit = MIN_TALLY_GAS_LIMIT;

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
        exec_inputs,
        exec_gas_limit,
        tally_program_id,
        tally_inputs,
        tally_gas_limit,
        memo: memo.as_slice().into(),
        replication_factor,
        consensus_filter,
        gas_price,
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
        exec_inputs: dr_args.exec_inputs,
        exec_gas_limit: dr_args.exec_gas_limit,
        tally_program_id: dr_args.tally_program_id,
        tally_inputs: dr_args.tally_inputs,
        tally_gas_limit: dr_args.tally_gas_limit,
        memo: dr_args.memo,
        replication_factor: dr_args.replication_factor,
        consensus_filter: dr_args.consensus_filter,
        gas_price: dr_args.gas_price,
        seda_payload: seda_payload.into(),
        commits: Default::default(),
        reveals: Default::default(),
        payback_address: payback_address.into(),

        height,
    }
}

impl TestAccount {
    #[track_caller]
    pub fn post_data_request(
        &self,
        posted_dr: PostDataRequestArgs,
        seda_payload: Vec<u8>,
        payback_address: Vec<u8>,
        env_height: u64,
        funds: Option<u128>,
    ) -> Result<String, ContractError> {
        let msg = execute::post_request::Execute {
            posted_dr,
            seda_payload: seda_payload.into(),
            payback_address: payback_address.into(),
        }
        .into();

        if env_height < self.test_info.block_height() {
            panic!("Invalid Test: Cannot post a data request in the past");
        }

        // set the chain height... will effect the height in the dr for us to sign.
        self.test_info.set_block_height(env_height);

        // someone posts a data request
        let res: PostRequestResponsePayload =
            self.test_info
                .execute_with_funds(self, &msg, funds.unwrap_or(min_post_dr_cost()))?;
        assert_eq!(
            env_height, res.height,
            "chain height does not match data request height"
        );
        Ok(res.dr_id)
    }

    #[track_caller]
    pub fn can_executor_commit(&self, dr_id: &str, reveal_message: &RevealMessage) -> bool {
        let dr = self.get_data_request(dr_id).unwrap();
        let commitment = reveal_message.try_hash().unwrap().to_hex();

        let factory = execute::commit_result::Execute::factory(
            dr_id.to_string(),
            commitment.clone(),
            self.pub_key_hex(),
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            dr.height,
        );
        let proof = self.prove(factory.get_hash());

        self.test_info
            .query(query::QueryMsg::CanExecutorCommit {
                dr_id:      dr_id.to_string(),
                public_key: self.pub_key_hex(),
                commitment: commitment.to_string(),
                proof:      proof.to_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn commit_result(&self, dr_id: &str, reveal_message: &RevealMessage) -> Result<(), ContractError> {
        let dr = self.get_data_request(dr_id).unwrap();
        let commitment = reveal_message.try_hash().unwrap().to_hex();

        let factory = execute::commit_result::Execute::factory(
            dr_id.to_string(),
            commitment,
            self.pub_key_hex(),
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            dr.height,
        );
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn commit_result_wrong_height(&self, dr_id: &str, reveal_message: RevealMessage) -> Result<(), ContractError> {
        let dr = self.get_data_request(dr_id).unwrap();
        let commitment = reveal_message.try_hash().unwrap().to_hex();

        let factory = execute::commit_result::Execute::factory(
            dr_id.to_string(),
            commitment,
            self.pub_key_hex(),
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            dr.height.saturating_sub(3),
        );
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn can_executor_reveal(&self, dr_id: &str) -> bool {
        self.test_info
            .query(query::QueryMsg::CanExecutorReveal {
                dr_id:      dr_id.to_string(),
                public_key: self.pub_key_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn create_reveal_message(&self, reveal_body: RevealBody) -> RevealMessage {
        let reveal_body_hash = reveal_body.try_hash().unwrap();

        let factory = execute::reveal_result::Execute::factory(
            reveal_body,
            self.pub_key_hex(),
            vec![],
            vec![],
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            reveal_body_hash,
        );
        let proof = self.prove(factory.get_hash());
        factory.create_message(proof)
    }

    #[track_caller]
    pub fn reveal_result(&self, reveal_message: RevealMessage) -> Result<(), ContractError> {
        self.test_info.execute(self, &reveal_message.into())
    }

    #[track_caller]
    pub fn remove_data_request(
        &self,
        dr_id: String,
        msgs: Vec<DistributionMessage>,
    ) -> Result<Vec<(String, u8)>, ContractError> {
        let mut requests = HashMap::new();
        requests.insert(dr_id, msgs);
        let msg = sudo::remove_requests::Sudo { requests }.into();
        self.test_info.sudo(&msg)
    }

    #[track_caller]
    pub fn remove_data_requests(
        &self,
        requests: HashMap<String, Vec<DistributionMessage>>,
    ) -> Result<Vec<(String, u8)>, ContractError> {
        let msg = sudo::remove_requests::Sudo { requests }.into();
        self.test_info.sudo(&msg)
    }

    #[track_caller]
    pub fn get_data_request(&self, dr_id: &str) -> Option<DataRequest> {
        self.test_info
            .query(query::QueryMsg::GetDataRequest {
                dr_id: dr_id.to_string(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_commit(&self, dr_id: Hash, public_key: PublicKey) -> Option<Hash> {
        self.test_info
            .query(query::QueryMsg::GetDataRequestCommitment {
                dr_id:      dr_id.to_hex(),
                public_key: public_key.to_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_commits(&self, dr_id: Hash) -> HashMap<String, Hash> {
        self.test_info
            .query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    pub fn get_data_request_reveal(&self, dr_id: Hash, public_key: PublicKey) -> Option<RevealBody> {
        self.test_info
            .query(query::QueryMsg::GetDataRequestReveal {
                dr_id:      dr_id.to_hex(),
                public_key: public_key.to_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_request_reveals(&self, dr_id: Hash) -> HashMap<String, RevealBody> {
        self.test_info
            .query(query::QueryMsg::GetDataRequestCommitments { dr_id: dr_id.to_hex() })
            .unwrap()
    }

    #[track_caller]
    pub fn get_data_requests_by_status(
        &self,
        status: DataRequestStatus,
        last_seen_index: Option<LastSeenIndexKey>,
        limit: u32,
    ) -> GetDataRequestsByStatusResponse {
        self.test_info
            .query(query::QueryMsg::GetDataRequestsByStatus {
                status,
                last_seen_index,
                limit,
            })
            .unwrap()
    }

    #[track_caller]
    pub fn expire_data_requests(&self) -> Result<(), ContractError> {
        let msg = expire_data_requests::Sudo {}.into();
        self.test_info.sudo(&msg)
    }

    #[track_caller]
    pub fn set_dr_config(&self, dr_config: DrConfig) -> Result<(), ContractError> {
        let msg = execute::ExecuteMsg::SetTimeoutConfig(dr_config).into();
        self.test_info.execute(self, &msg)
    }
}
