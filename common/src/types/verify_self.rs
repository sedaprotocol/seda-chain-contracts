use crate::{crypto::verify_proof, error::Result, types::Hash};

pub(crate) trait VerifySelf {
    type Extra;

    fn proof(&self) -> Result<Vec<u8>>;

    fn msg_hash(&self, chain_id: &str, contract_addr: &str, extra: Self::Extra) -> Result<Hash>;

    fn verify_inner(&self, public_key: &[u8], chain_id: &str, contract_addr: &str, extra: Self::Extra) -> Result<()> {
        let msg_hash = self.msg_hash(chain_id, contract_addr, extra)?;
        verify_proof(public_key, &self.proof()?, msg_hash)
    }
}
