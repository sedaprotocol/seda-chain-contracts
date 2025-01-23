use std::ops::Deref;

use common_types::ToHexStr;
use cw_storage_plus::PrimaryKey;
use error::ContractError;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey(pub [u8; 33]);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for PublicKey {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for PublicKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_hex_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromHexStr for PublicKey {
    fn from_hex_str(s: &str) -> Result<Self, ContractError> {
        let decoded = hex::decode(s)?;
        let array: [u8; 33] = decoded
            .try_into()
            .map_err(|d: Vec<u8>| ContractError::InvalidPublicKeyLength(d.len()))?;
        Ok(Self(array))
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = ContractError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 33 {
            return Err(ContractError::InvalidPublicKeyLength(value.len()));
        }
        let mut array = [0u8; 33];
        array.copy_from_slice(value);
        Ok(Self(array))
    }
}

impl PrimaryKey<'_> for PublicKey {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl ToHexStr for PublicKey {
    fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}

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

impl FromHexStr for Vec<u8> {
    fn from_hex_str(s: &str) -> Result<Self, ContractError> {
        hex::decode(s).map_err(ContractError::from)
    }
}
