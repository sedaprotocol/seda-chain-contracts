use error::ContractError;

use super::*;

pub type PublicKey = [u8; 33];

pub trait FromHexStr: Sized {
    fn from_hex_str(s: &str) -> Result<Self, ContractError>;
}

impl FromHexStr for common_types::Hash {
    fn from_hex_str(s: &str) -> Result<Self, ContractError> {
        let decoded = hex::decode(s)?;
        let array = decoded
            .try_into()
            .map_err(|d: Vec<u8>| ContractError::InvalidHashLength(d.len()))?;
        Ok(array)
    }
}

impl FromHexStr for PublicKey {
    fn from_hex_str(s: &str) -> Result<Self, ContractError> {
        let decoded = hex::decode(s)?;
        let array = decoded
            .try_into()
            .map_err(|d: Vec<u8>| ContractError::InvalidPublicKeyLength(d.len()))?;
        Ok(array)
    }
}

impl FromHexStr for Vec<u8> {
    fn from_hex_str(s: &str) -> Result<Self, ContractError> {
        hex::decode(s).map_err(ContractError::from)
    }
}
