use super::*;
use crate::state::CHAIN_ID;

impl ExecuteHandler for execute::reveal_result::Execute {
    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in
    /// the data results.
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // find the data request from the committed pool (if it exists, otherwise error)
        let dr_id_str = self.reveal_body.dr_id.clone();
        let dr_id = Hash::from_hex_str(&dr_id_str)?;
        let mut dr = state::load_request(deps.storage, &dr_id)?;

        // error if reveal phase for this DR has not started (i.e. replication factor is
        // not met)
        if !dr.base.reveal_started() {
            return Err(ContractError::RevealNotStarted);
        }

        // error if the data request has expired
        let expires_at = state::get_dr_expiration_height(deps.storage, &dr_id)?;
        if expires_at <= env.block.height {
            return Err(ContractError::DataRequestExpired(expires_at, "reveal"));
        }

        let chain_id = CHAIN_ID.load(deps.storage)?;
        let public_key = PublicKey::from_hex_str(&self.public_key)?;

        // error if data request executor has not submitted a commitment
        let Some(committed_dr_result) = dr.base.get_commitment(&self.public_key) else {
            return Err(ContractError::NotCommitted);
        };

        // error if data request executor has already submitted a reveal
        if dr.has_revealer(&self.public_key) {
            return Err(ContractError::AlreadyRevealed);
        }

        // error if the commitment hash does not match the reveal
        let expected_commitment = self.try_hash()?;
        if &expected_commitment != committed_dr_result {
            return Err(ContractError::RevealMismatch);
        }

        // check if the proxy_public_keys are valid
        for proxy in self.reveal_body.proxy_public_keys.iter() {
            PublicKey::from_hex_str(proxy)?;
        }

        // verify the proof
        let reveal_body_hash = self.reveal_body.try_hash()?;
        self.verify(
            public_key.as_ref(),
            &chain_id,
            env.contract.address.as_str(),
            reveal_body_hash,
        )?;

        let response = Response::new().add_attribute("action", "reveal_data_result").add_event(
            Event::new("seda-reveal").add_attributes([
                ("dr_id", dr_id_str.clone()),
                ("posted_dr_height", dr.base.height.to_string()),
                ("reveal", to_json_string(&self.reveal_body)?),
                ("stdout", to_json_string(&self.stdout)?),
                ("stderr", to_json_string(&self.stderr)?),
                ("executor", self.public_key.to_string()),
                ("version", CONTRACT_VERSION.to_string()),
            ]),
        );

        // add the reveal to the data request state
        dr.reveals.insert(self.public_key.clone());
        state::reveal(
            deps.storage,
            &dr_id,
            dr,
            env.block.height,
            &self.public_key,
            self.reveal_body,
        )?;

        Ok(response.add_message(new_refund_msg(env, dr_id_str, self.public_key, true)?))
    }
}
