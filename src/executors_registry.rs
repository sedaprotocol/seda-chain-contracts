#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::consts::MINIMUM_STAKE_TO_REGISTER;
use crate::helpers::get_attached_funds;
use crate::state::ELIGIBLE_DATA_REQUEST_EXECUTORS;
use crate::state::INACTIVE_DATA_REQUEST_EXECUTORS;
use crate::state::TOKEN;

use crate::msg::GetDataRequestExecutorResponse;
use crate::state::DataRequestExecutor;
use crate::ContractError;

pub mod data_request_executors {
    use cosmwasm_std::Addr;

    use crate::consts::MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY;

    use super::*;

    /// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        p2p_multi_address: Option<String>,
    ) -> Result<Response, ContractError> {
        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, token)?;

        if amount < MINIMUM_STAKE_TO_REGISTER {
            return Err(ContractError::InsufficientFunds(
                MINIMUM_STAKE_TO_REGISTER,
                amount,
            ));
        }

        let executor = DataRequestExecutor {
            p2p_multi_address: p2p_multi_address.clone(),
            tokens_staked: amount,
            tokens_pending_withdrawal: 0,
        };
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;

        if amount >= MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY {
            ELIGIBLE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;
        }
        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_attribute("executor", info.sender)
            .add_attribute("p2p_multi_address", p2p_multi_address.unwrap_or_default()))
    }

    /// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
    pub fn unregister_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // require that the executor has no staked or tokens pending withdrawal
        let executor = INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.sender.clone())?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        INACTIVE_DATA_REQUEST_EXECUTORS.remove(deps.storage, info.sender.clone());

        Ok(Response::new()
            .add_attribute("action", "unregister_data_request_executor")
            .add_attribute("executor", info.sender))
    }

    /// Returns a data request executor from the inactive executors with the given address, if it exists.
    pub fn get_data_request_executor(
        deps: Deps,
        executor: Addr,
    ) -> StdResult<GetDataRequestExecutorResponse> {
        let executor = INACTIVE_DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
        Ok(GetDataRequestExecutorResponse { value: executor })
    }
}

#[cfg(test)]
mod executers_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn register_data_request_executor() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetching data request executor for an address that doesn't exist should return None
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(value, GetDataRequestExecutorResponse { value: None });

        // someone registers a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch the data request executor
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 0
                })
            }
        );
    }

    #[test]
    fn unregister_data_request_executor() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone registers a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should be able to fetch the data request executor
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(
            value,
            GetDataRequestExecutorResponse {
                value: Some(DataRequestExecutor {
                    p2p_multi_address: Some("address".to_string()),
                    tokens_staked: 2,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // unstake and withdraw all tokens
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake { amount: 2 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Withdraw { amount: 2 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // unregister the data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UnregisterDataRequestExecutor {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // fetching data request executor after unregistering should return None
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataRequestExecutor {
                executor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let value: GetDataRequestExecutorResponse = from_binary(&res).unwrap();
        assert_eq!(value, GetDataRequestExecutorResponse { value: None });
    }
}
