use super::*;
use crate::state::CHAIN_ID;

impl ExecuteHandler for execute::commit_result::Execute {
    /// Posts a data result of a data request with an attached hash of the answer and salt.
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let dr_id = Hash::from_hex_str(&self.dr_id)?;
        let mut dr = state::load_request(deps.storage, &dr_id)?;

        // error if the user has already committed
        if dr.has_committer(&self.public_key) {
            return Err(ContractError::AlreadyCommitted);
        }

        // error if reveal stage has started (replication factor reached)
        if dr.reveal_started() {
            return Err(ContractError::RevealStarted);
        }

        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        // compute message hash
        let message_hash = hash([
            "commit_data_result".as_bytes(),
            self.dr_id.as_bytes(),
            &dr.height.to_be_bytes(),
            self.commitment.as_bytes(),
            chain_id.as_bytes(),
            env.contract.address.as_str().as_bytes(),
        ]);

        // verify the proof
        let proof = Vec::<u8>::from_hex_str(&self.proof)?;
        verify_proof(&public_key, &proof, message_hash)?;

        // add the commitment to the data request
        let commitment = Hash::from_hex_str(&self.commitment)?;
        dr.commits.insert(self.public_key.clone(), commitment);
        state::commit(deps.storage, &dr_id, dr)?;

        Ok(Response::new().add_attribute("action", "commit_data_result").add_event(
            Event::new("seda-commitment").add_attributes([
                ("dr_id", self.dr_id),
                ("commitment", self.commitment),
                ("executor", self.public_key),
                ("version", CONTRACT_VERSION.to_string()),
            ]),
        ))
    }
}
