use semver::Version;
use sha3::{Digest, Keccak256};

// pub type Bytes = Vec<u8>;
// pub type Commitment = Hash;
// pub type Memo = Vec<u8>;
pub type Hash = [u8; 32];
pub type PublicKey = Vec<u8>;

pub trait SimpleHash {
    fn simple_hash(&self) -> Hash;
}

impl SimpleHash for &str {
    fn simple_hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
}

impl SimpleHash for String {
    fn simple_hash(&self) -> Hash {
        let refer: &str = self.as_ref();
        refer.simple_hash()
    }
}

impl SimpleHash for Version {
    fn simple_hash(&self) -> Hash {
        self.to_string().simple_hash()
    }
}
