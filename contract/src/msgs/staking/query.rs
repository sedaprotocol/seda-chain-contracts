use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::Secp256k1PublicKey;

#[cw_serde]
#[derive(QueryResponses)]

pub enum QueryMsg {
    #[returns(Secp256k1PublicKey)]
    GetStaker { executor: Secp256k1PublicKey },
    #[returns(bool)]
    IsExecutorEligible { executor: Secp256k1PublicKey },
    #[returns(crate::state::StakingConfig)]
    GetStakingConfig,
}
