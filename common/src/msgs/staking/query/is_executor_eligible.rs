use core::str;

use crate::{error::Result, types::*};

#[cfg_attr(feature = "cosmwasm", cosmwasm_schema::cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(serde::Serialize, Debug, PartialEq))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub struct Query {
    pub data: Bytes,
}

impl Query {
    fn generate_hash(dr_id: &str, chain_id: &str, contract_addr: &str) -> Hash {
        crate::crypto::hash([
            "is_executor_eligible".as_bytes(),
            dr_id.as_bytes(),
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
        ])
    }

    #[cfg(not(feature = "cosmwasm"))]
    fn decode(&self) -> Result<Vec<u8>> {
        use base64::{prelude::BASE64_STANDARD, Engine};

        let decoded = BASE64_STANDARD.decode(&self.data)?;
        Ok(decoded)
    }

    #[cfg(feature = "cosmwasm")]
    fn decode(&self) -> Result<Vec<u8>> {
        Ok(self.data.to_vec())
    }

    fn dr_id_hex(&self) -> Result<String> {
        let decoded = self.decode()?;
        Ok(str::from_utf8(&decoded[67..131]).unwrap().to_owned())
    }

    pub fn parts(&self) -> Result<([u8; 33], Hash, Vec<u8>)> {
        let decoded = self.decode()?;
        let public_key = hex::decode(&decoded[..66])?.try_into().expect("Invalid public key");
        let dr_id = hex::decode(&decoded[67..131])?.try_into().expect("Invalid dr_id");
        let proof = hex::decode(&decoded[132..])?;

        Ok((public_key, dr_id, proof))
    }
}

impl VerifySelf for Query {
    type Extra = ();

    fn proof(&self) -> Result<Vec<u8>> {
        let decoded = self.decode()?;
        Ok(hex::decode(&decoded[132..])?)
    }

    fn msg_hash(&self, chain_id: &str, contract_addr: &str, _: Self::Extra) -> Result<Hash> {
        Ok(Self::generate_hash(&self.dr_id_hex()?, chain_id, contract_addr))
    }
}

pub struct QueryFactory {
    pub(crate) dr_id:      String,
    pub(crate) public_key: String,
    pub(crate) hash:       Hash,
}

impl QueryFactory {
    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }

    #[cfg(not(feature = "cosmwasm"))]
    pub(crate) fn encode_data(data: &str) -> Bytes {
        use base64::{prelude::BASE64_STANDARD, Engine};

        BASE64_STANDARD.encode(data)
    }

    #[cfg(feature = "cosmwasm")]
    pub(crate) fn encode_data(data: &str) -> Bytes {
        use cosmwasm_std::Binary;
        Binary::new(data.as_bytes().to_vec())
    }

    pub fn create_message(self, proof: Vec<u8>) -> (crate::msgs::QueryMsg, Bytes) {
        let data = format!("{}:{}:{}", self.public_key, self.dr_id, proof.to_hex());
        let base64_data = Self::encode_data(&data);

        (
            Query {
                data: base64_data.clone(),
            }
            .into(),
            base64_data,
        )
    }
}

impl Query {
    pub fn factory(public_key: String, dr_id: String, chain_id: &str, contract_addr: &str) -> QueryFactory {
        let hash = Self::generate_hash(&dr_id, chain_id, contract_addr);
        QueryFactory {
            dr_id,
            public_key,
            hash,
        }
    }

    pub fn verify(&self, public_key: &[u8], chain_id: &str, contract_addr: &str) -> Result<()> {
        self.verify_inner(public_key, chain_id, contract_addr, ())
    }
}

impl From<Query> for crate::msgs::QueryMsg {
    fn from(value: Query) -> Self {
        super::QueryMsg::IsExecutorEligible(value).into()
    }
}
