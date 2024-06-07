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

    fn public_key(&self) -> &[u8];
    fn proof(&self) -> &[u8];
    fn set_proof(&mut self, proof: Vec<u8>);

    // maybe needs to be an option if the struct has no fields
    fn fields(&self) -> impl IntoIterator<Item = &[u8]>;

    fn hash(&self, chain_id: &str, contract_addr: &str, seq: u128) -> Hash {
        let seq = seq.to_be_bytes();
        let msg = std::iter::once(Self::METHOD_NAME.as_bytes()).chain(self.fields().into_iter().chain([
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &seq,
        ]));

        hash(msg)
    }

    fn sign(&mut self, signing_key: &[u8], chain_id: &str, contract_addr: &str, seq: u128) -> Result<()> {
        let msg_hash = self.hash(chain_id, contract_addr, seq);

        let vrf = Secp256k1Sha256::default();
        let proof = vrf.prove(signing_key, &msg_hash)?;

        self.set_proof(proof);

        Ok(())
    }

    fn verify(&self, chain_id: &str, contract_addr: &str, seq: u128) -> Result<()> {
        verify_proof(self.public_key(), self.proof(), self.hash(chain_id, contract_addr, seq))
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

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    fn proof(&self) -> &[u8] {
        &self.proof
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

// Option 3: plain functions
impl Execute {
    #[allow(clippy::too_many_arguments)]
    pub fn new(dr_id: Hash, commitment: Hash, public_key: PublicKey) -> Result<Self> {
        Ok(Execute {
            dr_id,
            commitment,
            public_key,
            proof: vec![],
        })
    }

    pub fn hash(&self, height: u64, chain_id: &str, contract_addr: &str, seq: u128) -> Hash {
        hash([
            "commit_data_result".as_bytes(),
            &self.dr_id,
            // this one does expect a height... but I think this is wrong and we should remove that??
            &height.to_be_bytes(),
            &self.commitment,
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &seq.to_be_bytes(),
        ])
    }

    pub fn prove(
        &mut self,
        signing_key: &[u8],
        height: u64,
        chain_id: &str,
        contract_addr: &str,
        seq: u128,
    ) -> Result<()> {
        let msg_hash = self.hash(height, chain_id, contract_addr, seq);

        // We should lazy static or something to avoid creating a new instance every time
        let vrf = Secp256k1Sha256::default();
        let proof = vrf.prove(signing_key, &msg_hash)?;

        self.proof = proof;

        Ok(())
    }

    pub fn verify(&self, height: u64, chain_id: &str, contract_addr: &str, seq: u128) -> Result<()> {
        verify_proof(
            &self.public_key,
            &self.proof,
            self.hash(height, chain_id, contract_addr, seq),
        )
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::CommitDataResult(value).into()
    }
}
