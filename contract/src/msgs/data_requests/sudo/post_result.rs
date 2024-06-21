use super::*;

impl SudoHandler for sudo::post_result::Sudo {
    /// Posts a data request to the pool
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        // find the data request from the committed pool (if it exists, otherwise error)
        let dr_id = Hash::from_hex_str(&self.dr_id)?;
        let dr = state::load_request(deps.storage, &dr_id)?;

        if !dr.is_tallying() {
            return Err(ContractError::NotEnoughReveals);
        }

        let block_height: u64 = env.block.height;

        let event = Event::new("seda-data-post-result").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("dr_id", self.dr_id.clone()),
            ("block_height", block_height.to_string()),
            ("exit_code", self.exit_code.to_string()),
            ("result", to_json_string(&self.result)?),
            ("payback_address", dr.payback_address.to_base64()),
            ("seda_payload", dr.seda_payload.to_base64()),
        ]);

        state::post_result(deps.storage, &dr_id, &self.result)?;

        let result_id = self.result.try_hash()?;
        let event = event.add_attribute("result_id", result_id.to_hex());

        Ok(Response::new().add_event(event))
    }
}
