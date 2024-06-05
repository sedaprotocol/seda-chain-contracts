use super::{execute::*, *};
use crate::{
    crypto::hash,
    types::{Hasher, PublicKey},
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
    pub fn reg_and_stake(
        &mut self,
        sender: &mut TestExecutor,
        memo: Option<String>,
        amount: u128,
    ) -> Result<(), ContractError> {
        let msg_hash = hash(["register_and_stake".as_bytes(), &memo.hash()]);

        let msg = register_and_stake::Execute {
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
            memo,
        }
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn unregister(&mut self, sender: &TestExecutor) -> Result<(), ContractError> {
        let msg_hash = hash(["unregister".as_bytes()]);
        let msg = unregister::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
        }
        .into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn get_staker(&self, executor: PublicKey) -> Option<Staker> {
        self.query(query::QueryMsg::GetStaker { public_key: executor }).unwrap()
    }

    #[track_caller]
    pub fn increase_stake(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
        let msg_hash = hash(["increase_stake".as_bytes()]);
        let msg = increase_stake::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
        }
        .into();

        self.execute_with_funds(sender, &msg, amount)
    }

    #[track_caller]
    pub fn increase_stake_no_funds(&mut self, sender: &mut TestExecutor) -> Result<(), ContractError> {
        let msg_hash = hash(["increase_stake".as_bytes()]);
        let msg = increase_stake::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
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
        let msg = unstake::Execute {
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
        let msg = withdraw::Execute {
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
    pub fn get_account_sequence(&self, public_key: PublicKey) -> u64 {
        self.query(query::QueryMsg::GetAccountSeq { public_key }).unwrap()
    }
}
