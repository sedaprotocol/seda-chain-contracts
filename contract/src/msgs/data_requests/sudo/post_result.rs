use super::*;

#[cosmwasm_schema::cw_serde]
pub struct Sudo {
    pub(in crate::msgs::data_requests) dr_id:     Hash,
    pub(in crate::msgs::data_requests) result:    DataResult,
    pub(in crate::msgs::data_requests) exit_code: u8,
}

impl Sudo {
    /// Posts a data request to the pool
    pub fn execute(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        // find the data request from the committed pool (if it exists, otherwise error)
        let dr = state::load_request(deps.storage, &self.dr_id)?;

        if !dr.reveal_over() {
            return Err(ContractError::NotEnoughReveals);
        }

        let block_height: u64 = env.block.height;

        let event = Event::new("seda-data-post-result").add_attributes([
            ("version", CONTRACT_VERSION.to_string()),
            ("dr_id", self.dr_id.to_hex()),
            ("block_height", block_height.to_string()),
            ("exit_code", self.exit_code.to_string()),
            ("result", to_json_string(&self.result)?),
            ("payback_address", to_json_string(&dr.payback_address)?),
            ("seda_payload", to_json_string(&dr.seda_payload)?),
        ]);

        state::post_result(deps.storage, &self.dr_id, &self.result)?;

        let result_id = self.result.hash();
        let event = event.add_attribute("result_id", result_id.to_hex());

        Ok(Response::new().add_event(event))
    }
}

impl From<Sudo> for crate::msgs::SudoMsg {
    fn from(value: Sudo) -> Self {
        super::SudoMsg::PostDataResult(value).into()
    }
}
