use super::*;

#[cw_serde]
pub struct Execute {
    pub(in crate::msgs::data_requests) posted_dr:       PostDataRequestArgs,
    pub(in crate::msgs::data_requests) seda_payload:    Bytes,
    pub(in crate::msgs::data_requests) payback_address: Bytes,
}

impl Execute {
    /// Posts a data request to the pool
    pub fn execute(self, deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
        // hash the inputs to get the data request id
        let dr_id = self.posted_dr.hash();

        // require the data request id to be unique
        if state::data_request_or_result_exists(deps.as_ref(), dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // TODO: verify the payback address... it's not a seda addr right?
        // let Ok(addr) = Addr::from_slice(&self.payback_address) else {
        //     return Err(ContractError::InvalidPaybackAddr);
        // };

        // TODO: review this event
        let res = Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(to_json_binary(&dr_id)?)
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("dr_id", dr_id.to_hex()),
                ("dr_binary_id", self.posted_dr.dr_binary_id.to_hex()),
                ("tally_binary_id", self.posted_dr.tally_binary_id.to_hex()),
                ("dr_inputs", to_json_string(&self.posted_dr.dr_inputs)?),
                ("tally_inputs", to_json_string(&self.posted_dr.tally_inputs)?),
                ("memo", to_json_string(&self.posted_dr.memo)?),
                ("replication_factor", self.posted_dr.replication_factor.to_string()),
                ("gas_price", self.posted_dr.gas_price.to_string()),
                ("gas_limit", self.posted_dr.gas_limit.to_string()),
                ("seda_payload", to_json_string(&self.seda_payload)?),
                ("payback_address", to_json_string(&self.payback_address)?),
            ]));

        // save the data request
        let dr = DataRequest {
            id:                 dr_id,
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
        };
        state::insert_req(deps.storage, &dr_id, &dr)?;

        Ok(res)
    }
}

#[cfg(test)]
impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::PostDataRequest(value).into()
    }
}
