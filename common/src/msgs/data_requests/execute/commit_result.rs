use vrf_rs::Secp256k1Sha256;

use super::*;
use crate::error::Result;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Execute {
    pub dr_id:      Hash,
    pub commitment: Hash,
    pub public_key: PublicKey,
    pub proof:      Vec<u8>,
}

trait SignSelf {
    const METHOD_NAME: &'static str;

    fn set_proof(&mut self, proof: Vec<u8>);

    // maybe needs to be an option if the struct has no fields
    fn fields(&self) -> impl IntoIterator<Item = &[u8]>;

    fn sign(&mut self, signing_key: &[u8], chain_id: &str, contract_addr: &str, seq: u128) -> Result<()> {
        let seq = seq.to_be_bytes();
        let msg = std::iter::once(Self::METHOD_NAME.as_bytes()).chain(self.fields().into_iter().chain([
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &seq,
        ]));

        let msg_hash = hash(msg);

        let vrf = Secp256k1Sha256::default();
        let proof = vrf.prove(signing_key, &msg_hash)?;

        self.set_proof(proof);

        Ok(())
    }
}

// Option 1: Implement a trait for the structs
impl SignSelf for Execute {
    const METHOD_NAME: &'static str = "commit_data_result";

    fn set_proof(&mut self, proof: Vec<u8>) {
        self.proof = proof;
    }

    fn fields(&self) -> impl IntoIterator<Item = &[u8]> {
        [self.dr_id.as_slice(), self.commitment.as_slice()]
    }
}

// Option 2: Builder pattern
pub struct ExecuteBuilder {
    dr_id:      Hash,
    commitment: Hash,
    public_key: PublicKey,
}

impl ExecuteBuilder {
    pub fn new(dr_id: Hash, commitment: Hash, public_key: PublicKey) -> Self {
        ExecuteBuilder {
            dr_id,
            commitment,
            public_key,
        }
    }

    pub fn build(self, signing_key: &[u8], chain_id: &str, contract_addr: &str, seq: u128) -> Result<Execute> {
        let msg_hash = hash([
            "commit_data_result".as_bytes(),
            &self.dr_id,
            &self.commitment,
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &seq.to_be_bytes(),
        ]);

        let vrf = Secp256k1Sha256::default();

        let execute = Execute {
            dr_id:      self.dr_id,
            commitment: self.commitment,
            public_key: self.public_key,
            proof:      vrf.prove(signing_key, &msg_hash)?,
        };

        Ok(execute)
    }
}

// Option 3: long new function
impl Execute {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        signing_key: &[u8],
        dr_id: Hash,
        commitment: Hash,
        public_key: PublicKey,
        height: u64,
        chain_id: &str,
        contract_addr: &str,
        seq: u128,
    ) -> Result<Self> {
        let msg_hash = hash([
            "commit_data_result".as_bytes(),
            &dr_id,
            // this one does expect a height... but I think this is wrong and we should remove that??
            &height.to_be_bytes(),
            &commitment,
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &seq.to_be_bytes(),
        ]);

        // We should lazy static or something to avoid creating a new instance every time
        let vrf = Secp256k1Sha256::default();

        Ok(Execute {
            dr_id,
            commitment,
            public_key,
            proof: vrf.prove(signing_key, &msg_hash)?,
        })
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::CommitDataResult(value).into()
    }
}
