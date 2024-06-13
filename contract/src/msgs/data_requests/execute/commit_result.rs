use super::*;
use crate::state::{inc_get_seq, CHAIN_ID};

impl ExecuteHandler for execute::commit_result::Execute {
    /// Posts a data result of a data request with an attached hash of the answer and salt.
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let mut dr = state::load_req(deps.storage, &self.dr_id)?;

        // error if the user has already committed
        let public_key_str = hex::encode(&self.public_key);
        if dr.has_committer(&public_key_str) {
            return Err(ContractError::AlreadyCommitted);
        }

        // error if reveal stage has started (replication factor reached)
        if dr.reveal_started() {
            return Err(ContractError::RevealStarted);
        }

        let chain_id = CHAIN_ID.load(deps.storage)?;
        // compute message hash
        let message_hash = hash([
            "commit_data_result".as_bytes(),
            &self.dr_id,
            &dr.height.to_be_bytes(),
            &self.commitment,
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
            &inc_get_seq(deps.storage, &self.public_key)?.to_be_bytes(),
        ]);

        // verify the proof
        verify_proof(&self.public_key, &self.proof, message_hash)?;

        // add the commitment to the data request
        dr.commits.insert(public_key_str, self.commitment);
        state::commit(deps.storage, &self.dr_id, dr)?;

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
