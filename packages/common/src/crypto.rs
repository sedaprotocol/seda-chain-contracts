use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

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
