use crate::{error::Result, msgs::data_requests::RevealBody, types::*};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Execute {
    pub dr_id:       String,
    pub reveal_body: RevealBody,
    pub public_key:  String,
    pub proof:       String,
    pub stderr:      Vec<String>,
    pub stdout:      Vec<String>,
}

impl Execute {
    fn generate_hash(dr_id: &str, chain_id: &str, contract_addr: &str, dr_height: u64, reveal_body_hash: Hash) -> Hash {
        crate::crypto::hash([
            "reveal_data_result".as_bytes(),
            dr_id.as_bytes(),
            &dr_height.to_be_bytes(),
            &reveal_body_hash,
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
        ])
    }
}

impl VerifySelf for Execute {
    type Extra = (u64, Hash);

    fn proof(&self) -> Result<Vec<u8>> {
        Ok(hex::decode(&self.proof)?)
    }

    fn msg_hash(
        &self,
        chain_id: &str,
        contract_addr: &str,
        (dr_height, reveal_body_hash): Self::Extra,
    ) -> Result<Hash> {
        Ok(Self::generate_hash(
            &self.dr_id,
            chain_id,
            contract_addr,
            dr_height,
            reveal_body_hash,
        ))
    }
}

pub struct ExecuteFactory {
    dr_id:       String,
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

    pub fn create_message(self, proof: Vec<u8>) -> crate::msgs::ExecuteMsg {
        Execute {
            dr_id:       self.dr_id,
            reveal_body: self.reveal_body,
            public_key:  self.public_key,
            proof:       proof.to_hex(),
            stderr:      self.stderr,
            stdout:      self.stdout,
        }
        .into()
    }
}

impl Execute {
    #[allow(clippy::too_many_arguments)]
    pub fn factory(
        dr_id: String,
        reveal_body: RevealBody,
        public_key: String,
        stderr: Vec<String>,
        stdout: Vec<String>,
        chain_id: &str,
        contract_addr: &str,
        dr_height: u64,
        reveal_body_hash: Hash,
    ) -> ExecuteFactory {
        let hash = Self::generate_hash(&dr_id, chain_id, contract_addr, dr_height, reveal_body_hash);

        ExecuteFactory {
            dr_id,
            reveal_body,
            public_key,
            stderr,
            stdout,
            hash,
        }
    }

    pub fn verify(
        &self,
        public_key: &[u8],
        chain_id: &str,
        contract_addr: &str,
        dr_height: u64,
        reveal_body_hash: Hash,
    ) -> Result<()> {
        self.verify_inner(public_key, chain_id, contract_addr, (dr_height, reveal_body_hash))
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::RevealDataResult(value).into()
    }
}
