#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::consts::MINIMUM_STAKE_TO_REGISTER;
use crate::state::DATA_REQUEST_EXECUTORS;
use crate::state::TOKEN;
use crate::utils::{get_attached_funds, validate_sender};

use crate::error::ContractError;
use common::msg::GetDataRequestExecutorResponse;
use common::state::DataRequestExecutor;

pub mod data_request_executors {
    use cosmwasm_std::Addr;

    use crate::utils::apply_validator_eligibility;

    use super::*;

    /// Registers a data request executor with an optional p2p multi address, requiring a token deposit.
    pub fn register_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        p2p_multi_address: Option<String>,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require token deposit
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, &token)?;

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
        DATA_REQUEST_EXECUTORS.save(deps.storage, sender.clone(), &executor)?;

        apply_validator_eligibility(deps, sender.clone(), amount)?;

        Ok(Response::new()
            .add_attribute("action", "register_data_request_executor")
            .add_attribute("executor", sender)
            .add_attribute("p2p_multi_address", p2p_multi_address.unwrap_or_default()))
    }

    /// Unregisters a data request executor, with the requirement that no tokens are staked or pending withdrawal.
    pub fn unregister_data_request_executor(
        deps: DepsMut,
        info: MessageInfo,
        sender: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require that the executor has no staked or tokens pending withdrawal
        let executor = DATA_REQUEST_EXECUTORS.load(deps.storage, sender.clone())?;
        if executor.tokens_staked > 0 || executor.tokens_pending_withdrawal > 0 {
            return Err(ContractError::ExecutorHasTokens);
        }

        DATA_REQUEST_EXECUTORS.remove(deps.storage, sender.clone());

        Ok(Response::new()
            .add_attribute("action", "unregister_data_request_executor")
            .add_attribute("executor", sender))
    }

    /// Returns a data request executor from the inactive executors with the given address, if it exists.
    pub fn get_data_request_executor(
        deps: Deps,
        executor: Addr,
    ) -> StdResult<GetDataRequestExecutorResponse> {
        let executor = DATA_REQUEST_EXECUTORS.may_load(deps.storage, executor)?;
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
    use common::msg::{ExecuteMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn register_data_request_executor() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
            proxy: "proxy".to_string(),
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
            sender: None,
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
            proxy: "proxy".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone registers a data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
            sender: None,
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
        let msg = ExecuteMsg::Unstake {
            amount: 2,
            sender: None,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Withdraw {
            amount: 2,
            sender: None,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // unregister the data request executor
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UnregisterDataRequestExecutor { sender: None };
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
