use staking::state::STAKERS;
use state::{Escrow, DR_ESCROW};

use super::*;
use crate::{
    msgs::data_requests::{
        consts::{MIN_EXEC_GAS_LIMIT, MIN_GAS_PRICE, MIN_TALLY_GAS_LIMIT},
        state::DR_CONFIG,
    },
    state::TOKEN,
    utils::get_attached_funds,
};

impl ExecuteHandler for execute::post_request::Execute {
    /// Posts a data request to the pool
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // require the replication to be non-zero
        if self.posted_dr.replication_factor == 0 {
            return Err(ContractError::DataRequestReplicationFactorZero);
        }

        // require the gas price, and gas limits to be above the minimums
        if self.posted_dr.gas_price < MIN_GAS_PRICE {
            return Err(ContractError::GasPriceTooLow(self.posted_dr.gas_price));
        }
        if self.posted_dr.exec_gas_limit < MIN_EXEC_GAS_LIMIT {
            return Err(ContractError::ExecGasLimitTooLow(self.posted_dr.exec_gas_limit));
        }
        if self.posted_dr.tally_gas_limit < MIN_TALLY_GAS_LIMIT {
            return Err(ContractError::TallyGasLimitTooLow(self.posted_dr.tally_gas_limit));
        }
        // check the program ids are a valid hash length (32 bytes) in hex (64 utf-8
        // bytes)
        if self.posted_dr.exec_program_id.len() != 64 {
            return Err(ContractError::ProgramIdInvalidLength(
                "exec",
                self.posted_dr.exec_program_id.len(),
            ));
        }
        if self.posted_dr.tally_program_id.len() != 64 {
            return Err(ContractError::ProgramIdInvalidLength(
                "tally",
                self.posted_dr.tally_program_id.len(),
            ));
        }
        // check the size limits of the dr
        let dr_config = DR_CONFIG.load(deps.storage)?;
        if self.posted_dr.exec_inputs.len() > dr_config.exec_input_limit_in_bytes.get() as usize {
            return Err(ContractError::DrFieldTooBig(
                "exec inputs",
                self.posted_dr.exec_inputs.len(),
                dr_config.exec_input_limit_in_bytes,
            ));
        }
        if self.posted_dr.tally_inputs.len() > dr_config.tally_input_limit_in_bytes.get() as usize {
            return Err(ContractError::DrFieldTooBig(
                "tally inputs",
                self.posted_dr.tally_inputs.len(),
                dr_config.tally_input_limit_in_bytes,
            ));
        }
        if self.posted_dr.consensus_filter.len() > dr_config.consensus_filter_limit_in_bytes.get() as usize {
            return Err(ContractError::DrFieldTooBig(
                "consensus filter",
                self.posted_dr.consensus_filter.len(),
                dr_config.consensus_filter_limit_in_bytes,
            ));
        }
        if self.posted_dr.memo.len() > dr_config.memo_limit_in_bytes.get() as usize {
            return Err(ContractError::DrFieldTooBig(
                "memo",
                self.posted_dr.memo.len(),
                dr_config.memo_limit_in_bytes,
            ));
        }

        // require the data request replication factor to be bigger than amount of
        // stakers
        let stakers_length = STAKERS.len(deps.storage)?;
        if self.posted_dr.replication_factor as u32 > stakers_length {
            return Err(ContractError::DataRequestReplicationFactorTooHigh(stakers_length));
        }

        // hash the inputs to get the data request id
        let dr_id = self.posted_dr.try_hash()?;

        // require the data request id to be unique
        if state::data_request_exists(deps.as_ref(), dr_id) {
            return Err(ContractError::DataRequestAlreadyExists);
        }

        // Take the funds from the user
        let token = TOKEN.load(deps.storage)?;
        let funds = cw_utils::must_pay(&info, &token)?;
        let required = (Uint128::from(self.posted_dr.exec_gas_limit) + Uint128::from(self.posted_dr.tally_gas_limit))
            .checked_mul(self.posted_dr.gas_price)?;
        if funds < required {
            return Err(ContractError::InsufficientFunds(
                required,
                get_attached_funds(&info.funds, &token)?,
            ));
        };

        let dr_poster = info.sender.to_string();
        DR_ESCROW.save(
            deps.storage,
            &dr_id,
            &Escrow {
                amount: funds,
                poster: info.sender,
            },
        )?;

        // TODO: verify the payback non seda address...
        let hex_dr_id = dr_id.to_hex();
        let res = Response::new()
            .add_attribute("action", "post_data_request")
            .set_data(to_json_binary(&PostRequestResponsePayload {
                dr_id:  hex_dr_id.clone(),
                height: env.block.height,
            })?)
            .add_event(Event::new("seda-data-request").add_attributes([
                ("dr_id", hex_dr_id.clone()),
                ("dr_poster", dr_poster),
                ("exec_program_id", self.posted_dr.exec_program_id.clone()),
                ("exec_inputs", self.posted_dr.exec_inputs.to_base64()),
                ("exec_gas_limit", self.posted_dr.exec_gas_limit.to_string()),
                ("tally_program_id", self.posted_dr.tally_program_id.clone()),
                ("tally_inputs", self.posted_dr.tally_inputs.to_base64()),
                ("tally_gas_limit", self.posted_dr.tally_gas_limit.to_string()),
                ("replication_factor", self.posted_dr.replication_factor.to_string()),
                ("consensus_filter", self.posted_dr.consensus_filter.to_base64()),
                ("gas_price", self.posted_dr.gas_price.to_string()),
                ("memo", self.posted_dr.memo.to_base64()),
                ("seda_payload", self.seda_payload.to_base64()),
                ("payback_address", self.payback_address.to_base64()),
                ("version", self.posted_dr.version.to_string()),
            ]));

        // save the data request
        let dr = DataRequestContract {
            base:    DataRequestBase {
                id:                 hex_dr_id,
                version:            self.posted_dr.version,
                exec_program_id:    self.posted_dr.exec_program_id,
                exec_inputs:        self.posted_dr.exec_inputs,
                exec_gas_limit:     self.posted_dr.exec_gas_limit,
                tally_program_id:   self.posted_dr.tally_program_id,
                tally_inputs:       self.posted_dr.tally_inputs,
                tally_gas_limit:    self.posted_dr.tally_gas_limit,
                replication_factor: self.posted_dr.replication_factor,
                consensus_filter:   self.posted_dr.consensus_filter,
                gas_price:          self.posted_dr.gas_price,
                memo:               self.posted_dr.memo,

                payback_address: self.payback_address,
                seda_payload:    self.seda_payload,
                commits:         Default::default(),

                height: env.block.height,
            },
            reveals: Default::default(),
        };
        state::post_request(deps.storage, env.block.height, &dr_id, dr)?;

        Ok(res)
    }
}
