use common::crypto::recover_pubkey;
use common::test_utils::TestExecutor;
use sha3::{Digest, Keccak256};

#[test]
pub fn recover_pub_key_from_sig() {
    let executor = TestExecutor::new("test");

    let mut hasher = Keccak256::new();
    hasher.update("hello world".as_bytes());
    let hash = hasher.finalize();

    let sig = executor.sign(["hello world".as_bytes().to_vec()]);

    let pk = recover_pubkey(hash.into(), sig).unwrap();

    assert_eq!(pk, executor.public_key);
}
