#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::msg::GetDataResultResponse;
use crate::state::{DATA_REQUESTS_POOL, DATA_RESULTS};

use crate::state::DataResult;
use crate::ContractError;

pub mod data_request_results {

    use super::*;

    /// Posts a data result of a data request with an attached result.
    /// This removes the data request from the pool and creates a new entry in the data results.
    pub fn post_data_result(
        deps: DepsMut,
        _info: MessageInfo,
        dr_id: u128,
        result: String,
    ) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let dr = DATA_REQUESTS_POOL.load(deps.storage, dr_id)?;
        let dr_result = DataResult {
            dr_id: dr.dr_id,
            value: dr.value,
            result: result.clone(),
        };

        // save the data result then remove it from the pool
        DATA_RESULTS.save(deps.storage, dr_id, &dr_result)?;
        DATA_REQUESTS_POOL.remove(deps.storage, dr_id);

        Ok(Response::new()
            .add_attribute("action", "post_data_result")
            .add_attribute("dr_id", dr_id.to_string())
            .add_attribute("result", result))
    }

    /// Returns a data result from the results with the given id, if it exists.
    pub fn get_data_result(deps: Deps, dr_id: u128) -> StdResult<GetDataResultResponse> {
        let dr = DATA_RESULTS.may_load(deps.storage, dr_id)?;
        Ok(GetDataResultResponse { value: dr })
    }
}

#[cfg(test)]
mod dr_result_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::query;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    use crate::contract::instantiate;
    use crate::msg::GetDataRequestResponse;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn post_data_result() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // can't post a data result for a data request that doesn't exist
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: 0 as u128,
            result: "dr 0 result".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        // someone posts a data request
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataRequest {
            value: "hello world".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data result with id 0 does not yet exist
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataResult { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataRequestResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.value);

        // someone posts a data result
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostDataResult {
            dr_id: 0 as u128,
            result: "dr 0 result".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch data result with id 0
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataResult { dr_id: 0 },
        )
        .unwrap();
        let value: GetDataResultResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(DataResult {
                dr_id: 0 as u128,
                value: "hello world".to_string(),
                result: "dr 0 result".to_string()
            }),
            value.value
        );
    }
}