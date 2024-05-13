use super::helpers::instantiate_dr_contract;
use crate::contract::execute;
use common::msg::DataRequestsExecuteMsg;
use common::test_utils::TestExecutor;
use common::types::SimpleHash;
use cosmwasm_std::coins;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

#[test]
#[should_panic(expected = "NotProxy")]
fn only_proxy_can_pass_caller() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &coins(2, "token"));
    let exec = TestExecutor::new("creator");

    // instantiate contract
    instantiate_dr_contract(deps.as_mut(), info).unwrap();

    let dr_id = "dr_id".simple_hash();
    let commitment = "commitment".simple_hash();
    let sender = "someone".to_string();
    let signature = exec.sign([
        "commit_data_result".as_bytes().to_vec(),
        dr_id.to_vec(),
        commitment.to_vec(),
        sender.as_bytes().to_vec(),
    ]);

    // try commiting a data result from a non-proxy (doesn't matter if it's eligible or not since sender validation comes first)
    let msg = DataRequestsExecuteMsg::CommitDataResult {
        dr_id,
        commitment,
        sender: Some("someone".to_string()),
        signature,
    };
    let info = mock_info("anyone", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
}
