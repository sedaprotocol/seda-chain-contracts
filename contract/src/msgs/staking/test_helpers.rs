use seda_common::msgs::staking::{GetExecutorEligibilityResponse, GetExecutorsResponse};

use super::{
    msgs::staking::{execute, query},
    *,
};
use crate::TestAccount;

impl TestAccount {
    #[track_caller]
    pub fn set_staking_config(&self, config: StakingConfig) -> Result<(), ContractError> {
        let msg = config.into();
        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn stake(&self, amount: u128) -> Result<(), ContractError> {
        let memo = None;
        let seq = self.get_account_sequence();

        let factory = execute::stake::Execute::factory(
            self.pub_key_hex(),
            memo,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        )?;
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute_with_funds(self, &msg, amount)
    }

    #[track_caller]
    pub fn stake_with_memo(&self, amount: u128, memo: &'static str) -> Result<(), ContractError> {
        let memo = Some(Binary::from(memo.as_bytes()));
        let seq = self.get_account_sequence();

        let factory = execute::stake::Execute::factory(
            self.pub_key_hex(),
            memo,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        )?;
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute_with_funds(self, &msg, amount)
    }

    #[track_caller]
    pub fn get_staker_info(&self) -> Option<Staker> {
        self.test_info
            .query(query::QueryMsg::GetStaker {
                public_key: self.pub_key_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn increase_stake(&self, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence();

        let factory = execute::stake::Execute::factory(
            self.pub_key_hex(),
            None,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        )?;
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute_with_funds(self, &msg, amount)
    }

    #[track_caller]
    pub fn stake_with_no_funds(&self, memo: Option<String>) -> Result<(), ContractError> {
        let memo = memo.map(|s| Binary::from(s.as_bytes()));
        let seq = self.get_account_sequence();

        let factory = execute::stake::Execute::factory(
            self.pub_key_hex(),
            memo,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        )?;
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn unstake(&self) -> Result<(), ContractError> {
        let seq = self.get_account_sequence();

        let factory = execute::unstake::Execute::factory(
            self.pub_key_hex(),
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        );
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn withdraw(&self) -> Result<(), ContractError> {
        let seq = self.get_account_sequence();

        let factory = execute::withdraw::Execute::factory(
            self.pub_key_hex(),
            self.addr().to_string(),
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        );
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn withdraw_to(&self, withdraw_address: String) -> Result<(), ContractError> {
        let seq = self.get_account_sequence();

        let factory = execute::withdraw::Execute::factory(
            self.pub_key_hex(),
            withdraw_address,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
            seq,
        );
        let proof = self.prove(factory.get_hash());
        let msg = factory.create_message(proof);

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn is_staker_executor(&self) -> bool {
        self.test_info
            .query(query::QueryMsg::IsStakerExecutor {
                public_key: self.pub_key_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn is_executor_eligible(&self, dr_id: String) -> bool {
        let factory = query::is_executor_eligible::Query::factory(
            self.pub_key_hex(),
            dr_id,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
        );
        let proof = self.prove(factory.get_hash());
        let (query, _) = factory.create_message(proof);

        self.test_info.query(query).unwrap()
    }

    #[track_caller]
    pub fn is_executor_eligible_v2(&self, dr_id: String) -> GetExecutorEligibilityResponse {
        let factory = query::is_executor_eligible::Query::factory(
            self.pub_key_hex(),
            dr_id,
            self.test_info.chain_id(),
            self.test_info.contract_addr_str(),
        );
        let proof = self.prove(factory.get_hash());
        let (_, data) = factory.create_message(proof);
        let query_inner = query::is_executor_eligible::Query { data };

        self.test_info
            .query(query::QueryMsg::GetExecutorEligibility(query_inner))
            .unwrap()
    }

    #[track_caller]
    pub fn get_account_sequence(&self) -> Uint128 {
        self.test_info
            .query(query::QueryMsg::GetAccountSeq {
                public_key: self.pub_key_hex(),
            })
            .unwrap()
    }

    #[track_caller]
    pub fn query_executors(&self, offset: u32, limit: u32) -> GetExecutorsResponse {
        self.test_info
            .query(query::QueryMsg::GetExecutors { offset, limit })
            .unwrap()
    }
}
