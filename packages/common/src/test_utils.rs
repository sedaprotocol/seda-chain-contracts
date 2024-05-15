use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use sha3::{Digest, Keccak256};

use crate::types::{Hash, Secp256k1PublicKey, Signature};

pub struct TestExecutor {
    pub name:          &'static str,
    pub signing_key:   SigningKey,
    pub verifying_key: VerifyingKey,
    pub public_key:    Secp256k1PublicKey,
}

impl TestExecutor {
    pub fn new(name: &'static str) -> Self {
        let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
        let verifying_key = VerifyingKey::from(&signing_key);
        TestExecutor {
            name,
            signing_key,
            verifying_key,
            public_key: verifying_key.to_sec1_bytes().to_vec(),
        }
    }

    pub fn salt(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.name);
        hasher.finalize().into()
    }

    pub fn sign<I>(&self, msg: I) -> Signature
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        let mut hasher = Keccak256::new();
        for m in msg {
            hasher.update(m);
        }
        let hash = hasher.finalize();
        let (signature, rid) = self.signing_key.sign_recoverable(hash.as_ref()).unwrap();

        let mut sig: [u8; 65] = [0; 65];
        sig[0..64].copy_from_slice(&signature.to_bytes());
        sig[64] = rid.into();
        sig.into()
    }
}
