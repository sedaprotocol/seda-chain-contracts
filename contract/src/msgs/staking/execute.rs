use cosmwasm_schema::cw_serde;

use crate::{state::StakingConfig, types::Signature};

#[cw_serde]
pub enum ExecuteMsg {
    // staking msgs
    RegisterAndStake {
        signature: Signature,
        memo:      Option<String>,
    },
    Unregister {
        signature: Signature,
    },
    IncreaseStake {
        signature: Signature,
    },
    Unstake {
        signature: Signature,
        amount:    u128,
    },
    Withdraw {
        signature: Signature,
        amount:    u128,
    },
    SetStakingConfig {
        config: StakingConfig,
    },
}
