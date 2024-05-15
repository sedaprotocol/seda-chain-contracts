use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::{
    error::ContractError,
    types::{Secp256k1PublicKey, Signature as Sig},
};

pub fn recover_pubkey(msg_hash: [u8; 32], signature: Sig) -> Result<Secp256k1PublicKey, ContractError> {
    let rs = signature.sig_bytes().into();
    let id = match signature.rid() {
        0 => RecoveryId::new(false, false),
        1 => RecoveryId::new(true, false),
        _ => return Err(ContractError::InvalidSignatureRecoveryId),
    };

    let sig = Signature::from_bytes(rs).map_err(|_| ContractError::InvalidSignature)?;

    // Recover
    let pubkey = VerifyingKey::recover_from_msg(&msg_hash, &sig, id).map_err(|_| ContractError::InvalidSignature)?;
    // Safe to unwrap as we know the size of the compressed public key
    let compressed: [u8; 33] = pubkey.to_encoded_point(true).as_bytes().try_into().unwrap();
    Ok(compressed.into())
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
