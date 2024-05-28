use super::{state::DATA_REQUESTS, *};
use crate::crypto::{hash, verify_proof};

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::data_requests) dr_id:      Hash,
    pub(in crate::msgs::data_requests) commitment: Hash,
    pub(in crate::msgs::data_requests) public_key: PublicKey,
    pub(in crate::msgs::data_requests) proof:      Vec<u8>,
}

impl Execute {
    /// Posts a data result of a data request with an attached hash of the answer and salt.
    pub fn execute(self, deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        // compute message hash
        let message_hash = hash(["commit_data_result".as_bytes(), &self.dr_id, &self.commitment]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // find the data request from the pool (if it exists, otherwise error)
        let public_key_str = hex::encode(&self.public_key);
        let mut dr = DATA_REQUESTS.load(deps.storage, &self.dr_id)?;
        if dr.commits.contains_key(&public_key_str) {
            return Err(ContractError::AlreadyCommitted);
        }

        // error if reveal stage has started (replication factor reached)
        if u16::try_from(dr.commits.len()).unwrap() >= dr.replication_factor {
            return Err(ContractError::RevealStarted);
        }

        // add the commitment to the data request
        dr.commits.insert(public_key_str, self.commitment);
        // TODO: update the data request status
        DATA_REQUESTS.save(deps.storage, &self.dr_id, &dr)?;

        Ok(Response::new().add_attribute("action", "commit_data_result").add_event(
            Event::new("seda-commitment").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("dr_id", hex::encode(self.dr_id)),
                ("executor", info.sender.into_string()),
                ("commitment", hex::encode(self.commitment)),
            ]),
        ))
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::CommitDataResult(value).into()
    }
}
