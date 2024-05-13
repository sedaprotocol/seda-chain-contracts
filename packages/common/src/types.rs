use schemars::{
    gen::SchemaGenerator,
    schema::{Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use sha3::{Digest, Keccak256};

pub type Bytes = Vec<u8>;
pub type Commitment = Hash;
pub type Memo = Vec<u8>;
pub type Hash = [u8; 32];
pub type Secpk256k1PublicKey = Vec<u8>;

pub trait SimpleHash {
    fn simple_hash(&self) -> Hash;
}

impl SimpleHash for String {
    fn simple_hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
}

impl SimpleHash for Version {
    fn simple_hash(&self) -> Hash {
        self.to_string().simple_hash()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "BigArray")] pub(crate) [u8; 65]);

impl JsonSchema for Signature {
    fn schema_name() -> String {
        "Signature65".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::Array.into()),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items: Some(SingleOrVec::Single(Box::new(gen.subschema_for::<u8>()))),
                min_items: Some(65),
                max_items: Some(65),
                unique_items: Some(false),
                additional_items: None,
                contains: None,
            })),
            ..Default::default()
        };
        schema.into()
    }
}

impl From<[u8; 65]> for Signature {
    fn from(bytes: [u8; 65]) -> Self {
        Signature(bytes)
    }
}
