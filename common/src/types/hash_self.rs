use semver::Version;
use sha3::{Digest, Keccak256};

use crate::{error::Result, types::Hash};

pub trait TryHashSelf {
    fn try_hash(&self) -> Result<Hash>;
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

impl<T> HashSelf for &[T]
where
    T: HashSelf,
{
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        for item in *self {
            hasher.update(item.hash());
        }
        hasher.finalize().into()
    }
}

impl<const N: usize> HashSelf for [u8; N] {
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
    /// Hash the inner value if it exists, otherwise return default empty hash.
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        if let Some(inner) = self {
            hasher.update(inner);
        }
        hasher.finalize().into()
    }
}
