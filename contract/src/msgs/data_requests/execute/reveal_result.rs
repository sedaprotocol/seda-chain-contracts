use super::*;
use crate::state::CHAIN_ID;

impl ExecuteHandler for execute::reveal_result::Execute {
    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // find the data request from the committed pool (if it exists, otherwise error)
        let dr_id = Hash::from_hex_str(&self.dr_id)?;
        let mut dr = state::load_request(deps.storage, &dr_id)?;

        // error if reveal phase for this DR has not started (i.e. replication factor is not met)
        if !dr.reveal_started() {
            return Err(ContractError::RevealNotStarted);
        }

        // verify the proof
        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;
        let reveal_body_hash = self.reveal_body.try_hash()?;
        self.verify(
            &public_key,
            &chain_id,
            env.contract.address.as_str(),
            dr.height,
            reveal_body_hash,
        )?;

        // error if data request executor has not submitted a commitment
        let Some(committed_dr_result) = dr.get_commitment(&self.public_key) else {
            return Err(ContractError::NotCommitted);
        };

        // error if data request executor has already submitted a reveal
        if dr.has_revealer(&self.public_key) {
            return Err(ContractError::AlreadyRevealed);
        }

        // error if the commitment hash does not match the reveal
        // it's cheaper to hex -> byte array than hash -> hex
        dbg!(committed_dr_result, &reveal_body_hash);
        if &reveal_body_hash != committed_dr_result {
            return Err(ContractError::RevealMismatch);
        }

        let response = Response::new().add_attribute("action", "reveal_data_result").add_event(
            Event::new("seda-reveal").add_attributes([
                ("dr_id", self.dr_id.clone()),
                ("reveal", to_json_string(&self.reveal_body)?),
                ("stdout", to_json_string(&self.stdout)?),
                ("stderr", to_json_string(&self.stderr)?),
                ("executor", self.public_key.to_string()),
                ("version", CONTRACT_VERSION.to_string()),
            ]),
        );

        // add the reveal to the data request state
        dr.reveals.insert(self.public_key.clone(), self.reveal_body);
        state::reveal(deps.storage, &dr_id, dr)?;

        Ok(response)
    }
}
