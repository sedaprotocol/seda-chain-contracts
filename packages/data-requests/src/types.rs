use common::state::{DataResult, Reveal};
use common::types::Hash;
use serde::{Deserialize, Serialize};

pub type Input = Vec<u8>;
pub type PayloadItem = Vec<u8>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct CommitmentEntity {
    pub dr_id: Hash,
    pub executor: String,
    pub commitment: Hash,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RevealEntity {
    pub dr_id: Hash,
    pub executor: String,
    pub reveal: Reveal,
}
pub type DataResultEntity = Option<DataResult>;
