use crate::{error::Result, types::*};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Execute {
    pub public_key: String,
    pub proof:      String,
}

impl Execute {
    fn generate_hash(chain_id: &str, contract_addr: &str, sequence: U128) -> Hash {
        crate::crypto::hash([
            "unstake".as_bytes(),
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
            &sequence.to_be_bytes(),
        ])
    }
}

impl VerifySelf for Execute {
    type Extra = U128;

    fn proof(&self) -> Result<Vec<u8>> {
        Ok(hex::decode(&self.proof)?)
    }

    fn msg_hash(&self, chain_id: &str, contract_addr: &str, sequence: Self::Extra) -> Result<Hash> {
        Ok(Self::generate_hash(chain_id, contract_addr, sequence))
    }
}

pub struct ExecuteFactory {
    public_key: String,
    hash:       Hash,
}

impl ExecuteFactory {
    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn create_message(self, proof: Vec<u8>) -> crate::msgs::ExecuteMsg {
        Execute {
            public_key: self.public_key,
            proof:      proof.to_hex(),
        }
        .into()
    }
}

impl Execute {
    pub fn factory(public_key: String, chain_id: &str, contract_addr: &str, sequence: U128) -> ExecuteFactory {
        let hash = Self::generate_hash(chain_id, contract_addr, sequence);
        ExecuteFactory { public_key, hash }
    }

    pub fn verify(&self, public_key: &[u8], chain_id: &str, contract_addr: &str, sequence: U128) -> Result<()> {
        self.verify_inner(public_key, chain_id, contract_addr, sequence)
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::Unstake(value).into()
    }
}
