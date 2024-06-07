use super::*;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(QueryResponses))]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum QueryMsg {
    #[cfg_attr(feature = "cosmwasm", returns(Option<Staker>))]
    GetStaker { public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm", returns(U128))]
    GetAccountSeq { public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm", returns(StakerAndSeq))]
    GetStakerAndSeq { public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm", returns(bool))]
    IsExecutorEligible { public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm", returns(StakingConfig))]
    GetStakingConfig {},
}
