use seda_proto_common::prost::Message;

use super::{
    msgs::data_requests::execute::{self, ExecuteMsg},
    *,
};
use crate::state::PAUSED;

pub(in crate::msgs::data_requests) mod commit_result;
pub(crate) mod dr_events;
pub(in crate::msgs::data_requests) mod post_request;
pub(in crate::msgs::data_requests) mod reveal_result;
pub(in crate::msgs::data_requests) mod set_dr_config;

impl ExecuteHandler for ExecuteMsg {
    fn execute(self, deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // setting the timeout config is an owner operation and should not be paused
        if PAUSED.load(deps.storage)? && !matches!(self, ExecuteMsg::SetTimeoutConfig(_)) {
            return Err(ContractError::ContractPaused(
                "data request execute messages".to_string(),
            ));
        }

        match self {
            ExecuteMsg::CommitDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::PostDataRequest(msg) => msg.execute(deps, env, info),
            ExecuteMsg::RevealDataResult(msg) => msg.execute(deps, env, info),
            ExecuteMsg::SetTimeoutConfig(msg) => msg.execute(deps, env, info),
        }
    }
}

fn new_refund_msg(env: Env, dr_id: String, public_key: String, is_reveal: bool) -> Result<CosmosMsg, ContractError> {
    static TYPE_URL: &str = "/sedachain.wasm_storage.v1.MsgRefundTxFee";

    let refund_msg = seda_proto_common::wasm_storage::MsgRefundTxFee {
        authority: env.contract.address.to_string(),
        dr_id,
        public_key,
        is_reveal,
    };
    let mut vec = Vec::new();
    refund_msg.encode(&mut vec).map_err(ContractError::ProtoEncode)?;

    Ok(CosmosMsg::Any(AnyMsg {
        type_url: TYPE_URL.to_string(),
        value:    Binary::new(vec),
    }))
}
