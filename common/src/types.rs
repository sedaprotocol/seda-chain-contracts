#[cfg(feature = "cosmwasm")]
pub(crate) type U128 = cosmwasm_std::Uint128;
#[cfg(not(feature = "cosmwasm"))]
pub(crate) type U128 = String;

use semver::Version;
use sha3::{Digest, Keccak256};

pub type Bytes = Vec<u8>;
// pub type Commitment = Hash;
pub type Memo = Vec<u8>;
pub type Hash = [u8; 32];
pub type PublicKey = Vec<u8>;

pub trait ToHexStr: AsRef<[u8]> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl ToHexStr for Hash {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

pub trait HashSelf {
    fn hash(&self) -> Hash;
}

impl HashSelf for &str {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
}

impl HashSelf for String {
    fn hash(&self) -> Hash {
        let refer: &str = self.as_ref();
        refer.hash()
    }
}

impl HashSelf for Version {
    fn hash(&self) -> Hash {
        self.to_string().hash()
    }
}

impl HashSelf for Vec<u8> {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self);
        hasher.finalize().into()
    }
}

impl<T> HashSelf for Option<T>
where
    T: AsRef<[u8]>,
{
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        if let Some(inner) = self {
            hasher.update(inner);
        }
        hasher.finalize().into()
    }
}
