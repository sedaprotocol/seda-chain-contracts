use super::StakingConfig;

pub mod stake;
pub mod unstake;
pub mod withdraw;

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum ExecuteMsg {
    Stake(stake::Execute),
    Unstake(unstake::Execute),
    Withdraw(withdraw::Execute),
    SetStakingConfig(StakingConfig),
}

impl From<ExecuteMsg> for crate::msgs::ExecuteMsg {
    fn from(value: ExecuteMsg) -> Self {
        Self::Staking(value)
    }
}
