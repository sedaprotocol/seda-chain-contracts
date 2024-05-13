use common::{
    error::ContractError,
    msg::GetDataRequestExecutorResponse,
    state::DataRequestExecutor,
    test_utils::TestExecutor,
};
use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_info},
};

use super::helpers::*;

#[test]
fn register_data_request_executor() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    let exec = TestExecutor::new("anyone");
    // fetching data request executor for an address that doesn't exist should return None
    let value: GetDataRequestExecutorResponse = helper_get_executor(deps.as_mut(), exec.public_key.clone());

    assert_eq!(value, GetDataRequestExecutorResponse { value: None });

    // someone registers a data request executor
    let info = mock_info("anyone", &coins(2, "token"));

    let _res = helper_register_executor(deps.as_mut(), info, &exec, Some("memo".to_string()), None).unwrap();

    // should be able to fetch the data request executor

    let value: GetDataRequestExecutorResponse = helper_get_executor(deps.as_mut(), exec.public_key.clone());
    assert_eq!(
        value,
        GetDataRequestExecutorResponse {
            value: Some(DataRequestExecutor {
                memo:                      Some("memo".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 0,
            }),
        }
    );
}

#[test]
fn unregister_data_request_executor() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

    // someone registers a data request executor
    let info = mock_info("anyone", &coins(2, "token"));
    let exec = TestExecutor::new("anyone");

    let _res = helper_register_executor(deps.as_mut(), info, &exec, Some("memo".to_string()), None).unwrap();

    // should be able to fetch the data request executor
    let value: GetDataRequestExecutorResponse = helper_get_executor(deps.as_mut(), exec.public_key.clone());

    assert_eq!(
        value,
        GetDataRequestExecutorResponse {
            value: Some(DataRequestExecutor {
                memo:                      Some("memo".to_string()),
                tokens_staked:             2,
                tokens_pending_withdrawal: 0,
            }),
        }
    );

    // can't unregister the data request executor if it has staked tokens
    let info = mock_info("anyone", &coins(2, "token"));
    let res = helper_unregister_executor(deps.as_mut(), info, &exec, None);
    assert!(res.is_err_and(|x| x == ContractError::ExecutorHasTokens));

    // unstake and withdraw all tokens
    let info = mock_info("anyone", &coins(0, "token"));

    let _res = helper_unstake(deps.as_mut(), info.clone(), &exec, 2, None);
    let info = mock_info("anyone", &coins(0, "token"));
    let _res = helper_withdraw(deps.as_mut(), info.clone(), &exec, 2, None);

    // unregister the data request executor
    let info = mock_info("anyone", &coins(2, "token"));
    let _res = helper_unregister_executor(deps.as_mut(), info, &exec, None).unwrap();

    // fetching data request executor after unregistering should return None
    let value: GetDataRequestExecutorResponse = helper_get_executor(deps.as_mut(), exec.public_key.clone());

    assert_eq!(value, GetDataRequestExecutorResponse { value: None });
}
