use cosmwasm_std::{coins, testing::mock_info, MessageInfo};
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use sha3::{Digest, Keccak256};

use crate::{
    crypto::hash,
    types::{Hash, Secp256k1PublicKey, Signature},
};

pub struct TestExecutor {
    pub name:    &'static str,
    signing_key: SigningKey,
    public_key:  Secp256k1PublicKey,
    info:        MessageInfo,
}

impl TestExecutor {
    pub fn new(name: &'static str, amount: Option<u128>) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = verifying_key.to_encoded_point(true).to_bytes();
        let coins = if let Some(amount) = amount {
            coins(amount, "token")
        } else {
            vec![]
        };
        TestExecutor {
            name,
            signing_key,
            public_key: public_key.to_vec(),
            info: mock_info(name, &coins),
        }
    }

    pub fn pub_key(&self) -> Secp256k1PublicKey {
        self.public_key.clone()
    }

    pub fn info(&self) -> MessageInfo {
        self.info.clone()
    }

    pub fn set_amount(&mut self, amount: u128) {
        self.info = mock_info(self.name, &coins(amount, "token"));
    }

    pub fn _remove_coins(&mut self) {
        self.info = mock_info(self.name, &[]);
    }

    pub fn _salt(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.name);
        hasher.finalize().into()
    }

    pub fn sign<'a, I>(&self, msg: I) -> Signature
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let hash = hash(msg);
        let (signature, rid) = self.signing_key.sign_recoverable(hash.as_ref()).unwrap();

        let mut sig: [u8; 65] = [0; 65];
        sig[0..64].copy_from_slice(&signature.to_bytes());
        sig[64] = rid.into();
        sig.into()
    }
}
