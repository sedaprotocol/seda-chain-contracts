use cosmwasm_schema::cw_serde;

use super::StakingConfig;
use crate::types::PublicKey;

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
