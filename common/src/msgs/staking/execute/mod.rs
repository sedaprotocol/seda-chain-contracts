use super::*;

pub mod stake;
pub mod unstake;
pub mod withdraw;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum ExecuteMsg {
    Stake(stake::Execute),
    Unstake(unstake::Execute),
    Withdraw(withdraw::Execute),
    SetStakingConfig(StakingConfig),
}

impl From<ExecuteMsg> for super::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Staking(value)
    }
}
