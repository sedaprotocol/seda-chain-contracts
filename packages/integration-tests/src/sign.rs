use super::utils::TestExecutor;

use cosmwasm_crypto::secp256k1_recover_pubkey;
use k256::{ecdsa::VerifyingKey, EncodedPoint};
use sha3::{Digest, Keccak256};

#[test]
pub fn recover_pub_key_from_sig() {
    let mut executor = TestExecutor::new("test");

    let mut hasher = Keccak256::new();
    hasher.update("hello world".as_bytes());
    let hash = hasher.finalize();

    let (sig, rid) = executor.sign(&[&"hello world".as_bytes()]);
    dbg!(&rid);

    let encoded_point_bytes = secp256k1_recover_pubkey(&hash, &sig, rid.into()).unwrap();
    let encoded_point = EncodedPoint::from_bytes(encoded_point_bytes).unwrap();

    let vk = VerifyingKey::from_encoded_point(&encoded_point).unwrap();
    let pk = vk.to_sec1_bytes().to_vec();

    assert_eq!(pk, executor.public_key);
}
