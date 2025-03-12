mod to_hex;
pub use to_hex::ToHexStr;

mod hash_self;
pub use hash_self::{HashSelf, TryHashSelf};

#[cfg(not(feature = "cosmwasm"))]
mod uint128;

mod verify_self;
pub(crate) use verify_self::VerifySelf;

#[cfg(feature = "cosmwasm")]
pub(crate) type U128 = cosmwasm_std::Uint128;
#[cfg(not(feature = "cosmwasm"))]
pub(crate) type U128 = uint128::Uint128;

#[cfg(feature = "cosmwasm")]
pub(crate) type Bytes = cosmwasm_std::Binary;
#[cfg(not(feature = "cosmwasm"))]
pub(crate) type Bytes = String;

pub type Hash = [u8; 32];
