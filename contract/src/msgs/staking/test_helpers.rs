use super::{
    msgs::staking::{execute, query},
    *,
};
use crate::{
    crypto::hash,
    types::{HashSelf, PublicKey},
    TestExecutor,
    TestInfo,
};

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
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "stake".as_bytes(),
            &memo.hash(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);

        let msg = execute::stake::Execute {
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
            memo,
        }
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn get_staker(&self, executor: PublicKey) -> Option<Staker> {
        self.query(query::QueryMsg::GetStaker { public_key: executor }).unwrap()
    }

    #[track_caller]
    pub fn increase_stake(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "increase_stake".as_bytes(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);
        let msg = execute::stake::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
            memo:       None,
        }
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn stake_with_no_funds(
        &mut self,
        sender: &mut TestExecutor,
        memo: Option<String>,
    ) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "stake".as_bytes(),
            &memo.hash(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);
        let msg = execute::stake::Execute {
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
            memo,
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn unstake(&mut self, sender: &TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "unstake".as_bytes(),
            &amount.to_be_bytes(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);
        let msg = execute::unstake::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
            amount:     amount.into(),
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn withdraw(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
        let seq = self.get_account_sequence(sender.pub_key());
        let msg_hash = hash([
            "withdraw".as_bytes(),
            &amount.to_be_bytes(),
            self.chain_id(),
            self.contract_addr_bytes(),
            &seq.to_be_bytes(),
        ]);
        let msg = execute::withdraw::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
            amount:     amount.into(),
        }
        .into();

        let res = self.execute(sender, &msg);
        sender.add_seda(10);
        res
    }

    #[track_caller]
    pub fn is_executor_eligible(&self, executor: PublicKey) -> bool {
        self.query(query::QueryMsg::IsExecutorEligible { public_key: executor })
            .unwrap()
    }

    #[track_caller]
    pub fn get_account_sequence(&self, public_key: PublicKey) -> Uint128 {
        self.query(query::QueryMsg::GetAccountSeq { public_key }).unwrap()
    }
}
