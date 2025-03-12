use std::sync::OnceLock;

use sha3::{Digest, Keccak256};
use vrf_rs::Secp256k1Sha256;

use crate::{error::Result, types::Hash};

static VRF: OnceLock<Secp256k1Sha256> = OnceLock::new();

pub fn get_vrf() -> &'static Secp256k1Sha256 {
    VRF.get_or_init(Secp256k1Sha256::default)
}

pub fn verify_proof(public_key: &[u8], proof: &[u8], hash: Hash) -> Result<()> {
    let verified = get_vrf().verify(public_key, proof, &hash);

    // If we don't get an error it's always ok
    verified?;

    Ok(())
}

pub fn hash<'a, I>(iter: I) -> [u8; 32]
where
    I: IntoIterator<Item = &'a [u8]>,
{
    let mut hasher = Keccak256::new();
    for item in iter {
        hasher.update(item);
    }
    hasher.finalize().into()
}
