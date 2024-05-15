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
pub trait SimpleHash {
    fn simple_hash(&self) -> Hash;
}

impl SimpleHash for &str {
    fn simple_hash(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
}

impl SimpleHash for String {
    fn simple_hash(&self) -> Hash {
        let refer: &str = self.as_ref();
        refer.simple_hash()
    }
}

impl SimpleHash for Version {
    fn simple_hash(&self) -> Hash {
        self.to_string().simple_hash()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "BigArray")] [u8; 65]);

impl Signature {
    pub fn sig_bytes(&self) -> &[u8] {
        &self.0[0..64]
    }

    pub fn rid(&self) -> u8 {
        self.0[64]
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 65]> for Signature {
    fn from(bytes: [u8; 65]) -> Self {
        Self(bytes)
    }
}

impl JsonSchema for Signature {
    fn schema_name() -> String {
        "Signature65".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::Array.into()),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items:            Some(SingleOrVec::Single(Box::new(gen.subschema_for::<u8>()))),
                min_items:        Some(65),
                max_items:        Some(65),
                unique_items:     Some(false),
                additional_items: None,
                contains:         None,
            })),
            ..Default::default()
        };
        schema.into()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Secp256k1PublicKey(#[serde(with = "BigArray")] [u8; 33]);

impl AsRef<[u8]> for Secp256k1PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 33]> for Secp256k1PublicKey {
    fn from(bytes: [u8; 33]) -> Self {
        Self(bytes)
    }
}

impl JsonSchema for Secp256k1PublicKey {
    fn schema_name() -> String {
        "Secp256k1PublicKeyCompressed".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::Array.into()),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items:            Some(SingleOrVec::Single(Box::new(gen.subschema_for::<u8>()))),
                min_items:        Some(33),
                max_items:        Some(33),
                unique_items:     Some(false),
                additional_items: None,
                contains:         None,
            })),
            ..Default::default()
        };
        schema.into()
    }
}
