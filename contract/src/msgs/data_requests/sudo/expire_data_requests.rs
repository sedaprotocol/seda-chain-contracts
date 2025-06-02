use cosmwasm_std::{to_json_string, DepsMut, Env, Response, Uint128};
use seda_common::{
    msgs::data_requests::{sudo::expire_data_requests, DataRequestBase, DataRequestContract, RevealBody},
    types::{Hash, ToHexStr},
};
use semver::Version;

use super::{ContractError, SudoHandler};
use crate::{consts::INITIAL_DR_CONFIG, msgs::data_requests::state};

impl SudoHandler for expire_data_requests::Sudo {
    /// Expires all data requests that have timed out
    /// by moving them from whatever state they are in to the tallying state.
    fn sudo(self, deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let ids = state::expire_data_requests(deps.storage, env.block.height)?;

        let response = Response::new().add_attribute("method", "expire-data-requests");

        const DR_PER_BLOCK: u8 = 10;
        for i in 0..DR_PER_BLOCK {
            let mut dr_id: Hash = [0; 32];
            dr_id[0] = (env.block.height % 255) as u8;
            dr_id[1] = i as u8;
            let mut dr = DataRequestContract {
                base: DataRequestBase {
                    id: dr_id.to_hex(),
                    version: Version::new(0, 0, 0),
                    exec_program_id: "a".repeat(64).to_string(),
                    exec_inputs: "x"
                        .repeat(INITIAL_DR_CONFIG.exec_input_limit_in_bytes.get() as usize)
                        .as_bytes()
                        .into(),
                    exec_gas_limit: 0,
                    tally_program_id: "b".repeat(64).to_string(),
                    tally_inputs: "y".repeat(INITIAL_DR_CONFIG.tally_input_limit_in_bytes.get() as usize).as_bytes().into(),
                    tally_gas_limit: 0,
                    replication_factor: 1,
                    consensus_filter: "z".repeat(INITIAL_DR_CONFIG.consensus_filter_limit_in_bytes.get() as usize).as_bytes().into(),
                    gas_price: Uint128::from(0u128),
                    memo: "m".repeat(INITIAL_DR_CONFIG.memo_limit_in_bytes.get() as usize).as_bytes().into(),
                    payback_address: "o".repeat(INITIAL_DR_CONFIG.payback_address_limit_in_bytes.get() as usize).as_bytes().into(),
                    seda_payload: "p".repeat(INITIAL_DR_CONFIG.seda_payload_limit_in_bytes.get() as usize).as_bytes().into(),
                    commits: Default::default(),
                    height: 0,
                },
                reveals: Default::default(),
            };

            let identity = format!("{}33188bd0a0111b4de8da9ccab0225f72babcd0a46de83e0848f028f87a6efb14f", i);
            dr.base.commits.insert(identity.clone(), [0; 32]);
            dr.reveals.insert(identity.clone());
            state::post_request(deps.storage, env.block.height, &dr_id, dr.clone())?;

            state::reveal(
                deps.storage,
                &dr_id,
                dr,
                env.block.height,
                &identity,
                RevealBody {
                    dr_id: dr_id.to_hex(),
                    dr_block_height: env.block.height,
                    exit_code: 0,
                    gas_used: 0,
                    reveal: "g".repeat(INITIAL_DR_CONFIG.dr_reveal_size_limit_in_bytes.get() as usize).as_bytes().into(),
                    proxy_public_keys: vec![],
                },
            )?;
        }

        if ids.is_empty() {
            return Ok(response);
        }

        Ok(response.add_attribute("timed_out_drs", to_json_string(&ids)?))
    }
}
