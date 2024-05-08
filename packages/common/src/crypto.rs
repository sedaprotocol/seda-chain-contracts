use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::types::{Secpk256k1PublicKey, Signature as Sig};

pub fn recover_pubkey(msg_hash: [u8; 32], signature: Sig) -> Secpk256k1PublicKey {
    let rs = signature.0[0..64].into();
    let id = match signature.0[64] {
        0 => RecoveryId::new(false, false),
        1 => RecoveryId::new(true, false),
        _ => todo!("ERROR"),
    };

    let sig = Signature::from_bytes(rs).expect("TODO");

    // Recover
    let pubkey = VerifyingKey::recover_from_msg(&msg_hash, &sig, id).expect("TODO");
    pubkey.to_sec1_bytes().to_vec()
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
