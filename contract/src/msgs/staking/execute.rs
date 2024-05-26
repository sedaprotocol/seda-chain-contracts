use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use super::StakingConfig;
use crate::{config, error::ContractError, staking, types::PublicKey};

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAndStake {
        public_key: PublicKey,
        proof:      Vec<u8>,
        memo:       Option<String>,
    },
    Unregister {
        public_key: PublicKey,
        proof:      Vec<u8>,
    },
    IncreaseStake {
        public_key: PublicKey,
        proof:      Vec<u8>,
    },
    Unstake {
        public_key: PublicKey,
        proof:      Vec<u8>,
        amount:     u128,
    },
    Withdraw {
        public_key: PublicKey,
        proof:      Vec<u8>,
        amount:     u128,
    },
    SetStakingConfig {
        config: StakingConfig,
    },
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        use self::*;

        match self {
            ExecuteMsg::RegisterAndStake {
                public_key,
                proof,
                memo,
            } => staking::register_and_stake(deps, info, public_key, proof, memo),
            ExecuteMsg::IncreaseStake { public_key, proof } => {
                staking::increase_stake(deps, env, info, public_key, proof)
            }
            ExecuteMsg::Unstake {
                public_key,
                proof,
                amount,
            } => staking::unstake(deps, env, info, public_key, proof, amount),
            ExecuteMsg::Withdraw {
                public_key,
                proof,
                amount,
            } => staking::withdraw(deps, env, info, public_key, proof, amount),
            ExecuteMsg::Unregister { public_key, proof } => staking::unregister(deps, info, public_key, proof),
            ExecuteMsg::SetStakingConfig { config } => config::set_staking_config(deps, env, info, config),
        }
    }
}
