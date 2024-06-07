#[cfg(feature = "cosmwasm")]
use cosmwasm_std::Uint128;

pub type PublicKey = Vec<u8>;

#[cfg(feature = "cosmwasm")]
pub(crate) type U128 = Uint128;
#[cfg(not(feature = "cosmwasm"))]
pub(crate) type U128 = String;
