use super::*;

impl ExecuteHandler for execute::post_request::Execute {
    /// Posts a data request to the pool
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // hash the inputs to get the data request id
        let dr_id = self.posted_dr.hash();

        // require the data request id to be unique
        if state::data_request_or_result_exists(deps.as_ref(), dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // TODO: verify the payback non seda address...
        // TODO: review this event
        let hex_dr_id = dr_id.to_hex();
        let res = Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(to_json_binary(&dr_id)?)
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("dr_id", hex_dr_id.clone()),
                ("dr_binary_id", self.posted_dr.dr_binary_id.clone()),
                ("tally_binary_id", self.posted_dr.tally_binary_id.clone()),
                ("dr_inputs", self.posted_dr.dr_inputs.to_base64()),
                ("tally_inputs", self.posted_dr.tally_inputs.to_base64()),
                ("memo", self.posted_dr.memo.to_base64()),
                ("replication_factor", self.posted_dr.replication_factor.to_string()),
                ("gas_price", self.posted_dr.gas_price.to_string()),
                ("gas_limit", self.posted_dr.gas_limit.to_string()),
                ("seda_payload", self.seda_payload.to_base64()),
                ("payback_address", self.payback_address.to_base64()),
            ]));

        // save the data request
        let dr = DataRequest {
            id:                 hex_dr_id,
            version:            self.posted_dr.version,
            dr_binary_id:       self.posted_dr.dr_binary_id,
            dr_inputs:          self.posted_dr.dr_inputs,
            tally_binary_id:    self.posted_dr.tally_binary_id,
            tally_inputs:       self.posted_dr.tally_inputs,
            replication_factor: self.posted_dr.replication_factor,
            gas_price:          self.posted_dr.gas_price,
            gas_limit:          self.posted_dr.gas_limit,
            memo:               self.posted_dr.memo,

            payback_address: self.payback_address,
            seda_payload:    self.seda_payload,
            commits:         Default::default(),
            reveals:         Default::default(),

            height: env.block.height,
        };
        state::insert_req(deps.storage, &dr_id, dr)?;

        Ok(res)
    }
}
