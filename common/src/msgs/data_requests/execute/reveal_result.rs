use crate::{error::Result, msgs::data_requests::RevealBody, types::*};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Execute {
    pub reveal_body: RevealBody,
    pub public_key:  String,
    pub proof:       String,
    pub stderr:      Vec<String>,
    pub stdout:      Vec<String>,
}

impl Execute {
    fn generate_hash(chain_id: &str, contract_addr: &str, reveal_body_hash: Hash) -> Hash {
        crate::crypto::hash([
            "reveal_data_result".as_bytes(),
            &reveal_body_hash,
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
        ])
    }
}

impl VerifySelf for Execute {
    type Extra = Hash;

    fn proof(&self) -> Result<Vec<u8>> {
        Ok(hex::decode(&self.proof)?)
    }

    fn msg_hash(&self, chain_id: &str, contract_addr: &str, reveal_body_hash: Self::Extra) -> Result<Hash> {
        Ok(Self::generate_hash(chain_id, contract_addr, reveal_body_hash))
    }
}

impl TryHashSelf for Execute {
    fn try_hash(&self) -> Result<Hash> {
        let stderr = self.stderr.join("").into_bytes();
        let stdout = self.stdout.join("").into_bytes();

        Ok(crate::crypto::hash([
            "reveal_message".as_bytes(),
            self.reveal_body.try_hash()?.as_slice(),
            self.public_key.as_bytes(),
            self.proof.as_bytes(),
            &stderr,
            &stdout,
        ]))
    }
}

pub struct ExecuteFactory {
    reveal_body: RevealBody,
    public_key:  String,
    stderr:      Vec<String>,
    stdout:      Vec<String>,
    hash:        Hash,
}

impl ExecuteFactory {
    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn create_message(self, proof: Vec<u8>) -> Execute {
        Execute {
            reveal_body: self.reveal_body,
            public_key:  self.public_key,
            proof:       proof.to_hex(),
            stderr:      self.stderr,
            stdout:      self.stdout,
        }
    }
}

impl Execute {
    pub fn factory(
        reveal_body: RevealBody,
        public_key: String,
        stderr: Vec<String>,
        stdout: Vec<String>,
        chain_id: &str,
        contract_addr: &str,
        reveal_body_hash: Hash,
    ) -> ExecuteFactory {
        let hash = Self::generate_hash(chain_id, contract_addr, reveal_body_hash);

        ExecuteFactory {
            reveal_body,
            public_key,
            stderr,
            stdout,
            hash,
        }
    }

    pub fn verify(&self, public_key: &[u8], chain_id: &str, contract_addr: &str, reveal_body_hash: Hash) -> Result<()> {
        self.verify_inner(public_key, chain_id, contract_addr, reveal_body_hash)
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::RevealDataResult(value).into()
    }
}
