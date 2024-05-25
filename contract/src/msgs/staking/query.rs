use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::PublicKey;

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(PublicKey)]
    GetStaker { executor: PublicKey },
    #[returns(bool)]
    IsExecutorEligible { executor: PublicKey },
    #[returns(super::StakingConfig)]
    GetStakingConfig,
}
