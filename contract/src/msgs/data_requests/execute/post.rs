use cw_storage_plus::KeyDeserialize;

use super::{state::DATA_REQUESTS, utils::data_request_or_result_exists, *};

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
        if data_request_or_result_exists(deps.as_ref(), dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        let Ok(addr) = Addr::from_slice(&self.payback_address) else {
            return Err(ContractError::InvalidPaybackAddr);
        };

        // TODO: review this event
        let res = Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(Binary::from(dr_id))
            .add_event(Event::new("seda-data-request").add_attributes([
                ("version", CONTRACT_VERSION.to_string()),
                ("dr_id", dr_id.hash_hex()),
                ("dr_binary_id", self.posted_dr.dr_binary_id.hash_hex()),
                ("tally_binary_id", self.posted_dr.tally_binary_id.hash_hex()),
                // ("dr_inputs", &serde_json::to_string(&self.posted_dr.dr_inputs).unwrap()),
                // ("tally_inputs", &serde_json::to_string(&self.posted_dr.tally_inputs).unwrap()),
                // ("memo", &serde_json::to_string(&self.posted_dr.memo).unwrap()),
                ("replication_factor", self.posted_dr.replication_factor.to_string()),
                ("gas_price", self.posted_dr.gas_price.to_string()),
                ("gas_limit", self.posted_dr.gas_limit.to_string()),
                // ("seda_payload", &serde_json::to_string(&seda_payload).unwrap()),
                ("payback_address", addr.into_string()),
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
            commits:         HashMap::new(),
            reveals:         HashMap::new(),
        };
        DATA_REQUESTS.save(deps.storage, &dr_id, &dr)?;

        Ok(res)
    }
}

impl From<Execute> for crate::msgs::ExecuteMsg {
    fn from(value: Execute) -> Self {
        super::ExecuteMsg::PostDataRequest(value).into()
    }
}
