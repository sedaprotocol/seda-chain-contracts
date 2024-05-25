use common::{
    error::ContractError,
    msg::{DataRequestsExecuteMsg as ExecuteMsg, DataRequestsQueryMsg as QueryMsg, InstantiateMsg},
};
use cosmwasm_std::StdResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    data_request::data_requests,
    data_request_result::data_request_results,
    state::{DATA_REQUESTS_POOL, PROXY_CONTRACT, TOKEN},
};

// version info for migration info
const CONTRACT_NAME: &str = "data-requests";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DATA_REQUESTS_POOL.initialize(deps.storage)?;
    TOKEN.save(deps.storage, &msg.token)?;
    PROXY_CONTRACT.save(deps.storage, &deps.api.addr_validate(&msg.proxy)?)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostDataRequest {
            posted_dr,
            seda_payload,
            payback_address,
        } => data_requests::post_data_request(deps, info, posted_dr, seda_payload, payback_address),

        ExecuteMsg::CommitDataResult {
            dr_id,
            commitment,
            sender,
            signature,
        } => data_request_results::commit_result(deps, info, dr_id, commitment, sender, signature),
        ExecuteMsg::RevealDataResult {
            dr_id,
            reveal,
            signature,
            sender,
        } => data_request_results::reveal_result(deps, info, env, dr_id, reveal, signature, sender),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDataRequest { dr_id } => to_json_binary(&data_requests::get_data_request(deps, dr_id)?),
        QueryMsg::GetDataRequestsFromPool { position, limit } => {
            to_json_binary(&data_requests::get_data_requests_from_pool(deps, position, limit)?)
        }
        QueryMsg::GetCommittedDataResult { dr_id, executor } => {
            to_json_binary(&data_request_results::get_committed_data_result(deps, dr_id, executor)?)
        }
        QueryMsg::GetCommittedDataResults { dr_id } => {
            to_json_binary(&data_request_results::get_committed_data_results(deps, dr_id)?)
        }
        QueryMsg::GetRevealedDataResult { dr_id, executor } => {
            to_json_binary(&data_request_results::get_revealed_data_result(deps, dr_id, executor)?)
        }
        QueryMsg::GetRevealedDataResults { dr_id } => {
            to_json_binary(&data_request_results::get_revealed_data_results(deps, dr_id)?)
        }
        QueryMsg::GetResolvedDataResult { dr_id } => {
            to_json_binary(&data_request_results::get_resolved_data_result(deps, dr_id)?)
        }
    }
}

#[cfg(test)]
mod init_tests {

    use cosmwasm_std::{
        coins,
        testing::{mock_dependencies, mock_info},
    };

    use crate::test::helpers::instantiate_dr_contract;
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "token"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate_dr_contract(deps.as_mut(), info).unwrap();
        assert_eq!(0, res.messages.len());
    }
}