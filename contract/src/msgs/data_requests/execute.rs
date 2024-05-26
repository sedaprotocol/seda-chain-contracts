use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use super::PostDataRequestArgs;
use crate::{data_requests, error::ContractError, types::Bytes};

#[cw_serde]
pub enum ExecuteMsg {
    PostDataRequest {
        posted_dr:       PostDataRequestArgs,
        seda_payload:    Bytes,
        payback_address: Bytes,
    },
    // CommitDataResult {
    //     dr_id:      Hash,
    //     commitment: Hash,
    //     sender:     Option<String>,
    //     public_key: PublicKey,
    // },
    // RevealDataResult {
    //     dr_id:      Hash,
    //     reveal:     RevealBody,
    //     public_key: PublicKey,
    //     sender:     Option<String>,
    // },
}

impl ExecuteMsg {
    pub fn execute(self, deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        match self {
            ExecuteMsg::PostDataRequest {
                posted_dr,
                seda_payload,
                payback_address,
            } => data_requests::post_data_request(deps, info, posted_dr, seda_payload, payback_address),
        }
    }
}
