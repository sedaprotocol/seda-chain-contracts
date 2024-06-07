#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "cosmwasm"))]
use serde::Serialize;

use crate::msgs::staking::StakingConfig;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
pub struct Execute {
    pub config: StakingConfig,
}
