use semver::Version;
use sha3::{Digest, Keccak256};

pub type Bytes = Vec<u8>;
// pub type Commitment = Hash;
pub type Memo = Vec<u8>;
pub type Hash = [u8; 32];
pub type PublicKey = Vec<u8>;

pub trait Hasher {
    fn hash(&self) -> Hash;

    fn hash_hex(&self) -> String {
        hex::encode(self.hash())
    }
}

impl Hasher for &str {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
}

impl Hasher for String {
    fn hash(&self) -> Hash {
        let refer: &str = self.as_ref();
        refer.hash()
    }
}

impl Hasher for Version {
    fn hash(&self) -> Hash {
        self.to_string().hash()
    }
}

impl<const N: usize> Hasher for [u8; N] {
    fn hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(&self);
        hasher.finalize().into()
    }
}
