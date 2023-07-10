#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::state::INACTIVE_DATA_REQUEST_EXECUTORS;

use crate::error::ContractError;
use crate::helpers::get_attached_funds;
use crate::state::TOKEN;

#[allow(clippy::module_inception)]
pub mod staking {
    use cosmwasm_std::{coins, BankMsg};

    use super::*;

    /// Deposits and stakes tokens for a data request executor.
    pub fn deposit_and_stake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let amount = get_attached_funds(&info.funds, token)?;

        // update staked tokens for executor
        let mut executor =
            INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.clone().sender)?;
        executor.tokens_staked += amount;
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.clone().sender, &executor)?;

        Ok(Response::new()
            .add_attribute("action", "stake")
            .add_attribute("executor", info.sender)
            .add_attribute("amount", amount.to_string()))
    }

    /// Unstakes tokens to be withdrawn after a delay.
    pub fn unstake(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        amount: u128,
    ) -> Result<Response, ContractError> {
        // error if amount is greater than staked tokens
        let mut executor =
            INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.sender.clone())?;
        if amount > executor.tokens_staked {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_staked,
                amount,
            ));
        }

        // update the executor
        executor.tokens_staked -= amount;
        executor.tokens_pending_withdrawal += amount;
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;

        // TODO: emit when pending tokens can be withdrawn
        Ok(Response::new()
            .add_attribute("action", "unstake")
            .add_attribute("executor", info.sender)
            .add_attribute("amount", amount.to_string()))
    }

    /// Sends tokens back to the executor that are marked as pending withdrawal.
    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: u128,
    ) -> Result<Response, ContractError> {
        // TODO: add delay after calling unstake
        let token = TOKEN.load(deps.storage)?;

        // error if amount is greater than pending tokens
        let mut executor =
            INACTIVE_DATA_REQUEST_EXECUTORS.load(deps.storage, info.sender.clone())?;
        if amount > executor.tokens_pending_withdrawal {
            return Err(ContractError::InsufficientFunds(
                executor.tokens_pending_withdrawal,
                amount,
            ));
        }

        // update the executor
        executor.tokens_pending_withdrawal -= amount;
        INACTIVE_DATA_REQUEST_EXECUTORS.save(deps.storage, info.sender.clone(), &executor)?;

        // send the tokens back to the executor
        let bank_msg = BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: coins(amount, token),
        };

        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("executor", info.sender)
            .add_attribute("amount", amount.to_string()))
    }
}

#[cfg(test)]
mod staking_tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::msg::GetDataRequestExecutorResponse;
    use crate::msg::InstantiateMsg;
    use crate::msg::{ExecuteMsg, QueryMsg};
    use crate::state::DataRequestExecutor;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn deposit_stake_withdraw() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            token: "token".to_string(),
        };
        let info = mock_info("creator", &coins(0, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // cant register without depositing tokens
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::InsufficientFunds(1, 0));

        // register a data request executor
        let info = mock_info("anyone", &coins(1, "token"));
        let msg = ExecuteMsg::RegisterDataRequestExecutor {
            p2p_multi_address: Some("address".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 1
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
                    tokens_staked: 1,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor stakes 2 more tokens
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::DepositAndStake;
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 3
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
                    tokens_staked: 3,
                    tokens_pending_withdrawal: 0
                })
            }
        );

        // the data request executor unstakes 1
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Unstake { amount: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 1 and pending 1
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
                    tokens_pending_withdrawal: 1
                })
            }
        );

        // the data request executor withdraws 1
        let info = mock_info("anyone", &coins(0, "token"));
        let msg = ExecuteMsg::Withdraw { amount: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // data request executor's stake should be 1 and pending 0
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
}
