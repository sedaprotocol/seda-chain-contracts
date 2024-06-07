#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "cosmwasm"))]
use serde::Serialize;

use crate::types::{PublicKey, U128};

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
pub struct Execute {
    pub public_key: PublicKey,
    pub proof:      Vec<u8>,
    pub amount:     U128,
}
