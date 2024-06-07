use seda_contract_common::msgs::{
    self,
    staking::{query::QueryMsg, Staker, StakingConfig},
};

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
        let msg = crate::msgs::ExecuteMsg::Staking(
            seda_contract_common::msgs::staking::execute::ExecuteMsg::SetStakingConfig(config),
        );
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

        let msg = msgs::staking::execute::stake::Execute {
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
            memo,
        };
        // TODO: impl `From` trait
        let msg =
            crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Stake(msg));

        self.execute_with_funds(sender, &msg, amount)
    }

    // #[track_caller]
    // pub fn unregister(&mut self, sender: &TestExecutor) -> Result<(), ContractError> {
    //     let seq = self.get_account_sequence(sender.pub_key());
    //     let msg_hash = hash([
    //         "unregister".as_bytes(),
    //         self.chain_id(),
    //         self.contract_addr_bytes(),
    //         &seq.to_be_bytes(),
    //     ]);
    //     let msg = unregister::Execute {
    //         public_key: sender.pub_key(),
    //         proof:      sender.prove(&msg_hash),
    //     };
    //     let msg =
    //     crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Stake(msg));

    //     self.execute(sender, &msg)
    // }

    #[track_caller]
    pub fn get_staker(&self, executor: PublicKey) -> Option<Staker> {
        self.query(QueryMsg::GetStaker { public_key: executor }).unwrap()
    }

    // #[track_caller]
    // pub fn increase_stake(&mut self, sender: &mut TestExecutor, amount: u128) -> Result<(), ContractError> {
    //     let seq = self.get_account_sequence(sender.pub_key());
    //     let msg_hash = hash([
    //         "increase_stake".as_bytes(),
    //         self.chain_id(),
    //         self.contract_addr_bytes(),
    //         &seq.to_be_bytes(),
    //     ]);
    //     let msg = stake::Execute {
    //         public_key: sender.pub_key(),
    //         proof:      sender.prove(&msg_hash),
    //         memo: None,
    //     };
    //     let msg =
    //     crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Stake(msg));

    //     self.execute_with_funds(sender, &msg, amount)
    // }

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
        let msg = stake::Execute {
            public_key: sender.pub_key(),
            proof: sender.prove(&msg_hash),
            memo,
        };

        // TODO: impl `From` trait
        let msg =
            crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Stake(msg));

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
        let msg = msgs::staking::execute::unstake::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
            amount:     amount.into(),
        };

        // TODO: impl `From` trait
        let msg =
            crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Unstake(msg));

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
        let msg = msgs::staking::execute::withdraw::Execute {
            public_key: sender.pub_key(),
            proof:      sender.prove(&msg_hash),
            amount:     amount.into(),
        };

        // TODO: impl `From` trait
        let msg =
            crate::msgs::ExecuteMsg::Staking(seda_contract_common::msgs::staking::execute::ExecuteMsg::Withdraw(msg));

        let res = self.execute(sender, &msg);
        sender.add_seda(10);
        res
    }

    #[track_caller]
    pub fn is_executor_eligible(&self, executor: PublicKey) -> bool {
        self.query(QueryMsg::IsExecutorEligible { public_key: executor })
            .unwrap()
    }

    #[track_caller]
    pub fn get_account_sequence(&self, public_key: PublicKey) -> Uint128 {
        self.query(QueryMsg::GetAccountSeq { public_key }).unwrap()
    }
}
