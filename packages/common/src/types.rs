use schemars::{
    gen::SchemaGenerator,
    schema::{Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub type Bytes = Vec<u8>;
pub type Commitment = Hash;
pub type Memo = Vec<u8>;
pub type Hash = [u8; 32];
pub type Secpk256k1PublicKey = Vec<u8>;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(pub(crate) [u8; 65]);

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(65)?;
        for byte in &self.0 {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor;

        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array of length 65")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Signature, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut array = [0u8; 65];
                for (i, byte) in array.iter_mut().enumerate() {
                    *byte = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(Signature(array))
            }
        }

        deserializer.deserialize_tuple(65, ArrayVisitor)
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
