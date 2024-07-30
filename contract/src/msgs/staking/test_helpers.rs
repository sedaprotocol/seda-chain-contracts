use super::{
    msgs::staking::{execute, query},
    *,
};
use crate::{types::PublicKey, TestExecutor, TestInfo};

impl TestInfo {
    #[track_caller]
    pub fn set_staking_config(&mut self, sender: &TestExecutor, config: StakingConfig) -> Result<(), ContractError> {
        let msg = config.into();
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn stake(
        &mut self,
        sender: &mut TestExecutor,
        memo: Option<String>,
        amount: u128,
    ) -> Result<(), ContractError> {
        let memo = memo.map(|s| Binary::from(s.as_bytes()));
        let seq = self.get_account_sequence(sender.pub_key());

        let msg = execute::stake::Execute::new(
            sender.pub_key_hex(),
            memo,
            &sender.sign_key(),
            self.chain_id(),
            self.contract_addr(),
            seq.into(),
        )?
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn get_staker(&self, executor: PublicKey) -> Option<Staker> {
        self.query(query::QueryMsg::GetStaker {
            public_key: executor.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn increase_stake(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());

        let msg = execute::stake::Execute::new(
            sender.pub_key_hex(),
            None,
            &sender.sign_key(),
            self.chain_id(),
            self.contract_addr(),
            seq.into(),
        )?
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn stake_with_no_funds(
        &mut self,
        sender: &mut TestExecutor,
        memo: Option<String>,
    ) -> Result<(), ContractError> {
        let memo = memo.map(|s| Binary::from(s.as_bytes()));
        let seq = self.get_account_sequence(sender.pub_key());

        let msg = execute::stake::Execute::new(
            sender.pub_key_hex(),
            memo,
            &sender.sign_key(),
            self.chain_id(),
            self.contract_addr(),
            seq.into(),
        )?
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn unstake(&mut self, sender: &TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());

        let msg = execute::unstake::Execute::new(
            sender.pub_key_hex(),
            amount.into(),
            &sender.sign_key(),
            self.chain_id(),
            self.contract_addr(),
            seq.into(),
        )?
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn withdraw(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());

        let msg = execute::withdraw::Execute::new(
            sender.pub_key_hex(),
            amount.into(),
            &sender.sign_key(),
            self.chain_id(),
            self.contract_addr(),
            seq.into(),
        )?
        .into();

        let res = self.execute(sender, &msg);
        sender.add_seda(10);
        res
    }

    #[track_caller]
    pub fn is_executor_eligible(&self, executor: PublicKey) -> bool {
        self.query(query::QueryMsg::IsExecutorEligible {
            public_key: executor.to_hex(),
        })
        .unwrap()
    }

    #[track_caller]
    pub fn get_account_sequence(&self, public_key: PublicKey) -> Uint128 {
        self.query(query::QueryMsg::GetAccountSeq {
            public_key: public_key.to_hex(),
        })
        .unwrap()
    }
}
