#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "cosmwasm"))]
use serde::Serialize;

use crate::types::PublicKey;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
pub struct Execute {
    pub public_key: PublicKey,
    pub proof:      Vec<u8>,
    pub memo:       Option<String>,
}

impl From<Execute> for super::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Stake(value)
    }
}
