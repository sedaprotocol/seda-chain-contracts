use seda_common::types::TryHashSelf;

use crate::msgs::data_requests::test_helpers;

mod commit_dr;
mod pause_behavior;
mod post_dr;
mod query_dr_status;
mod remove_dr;
mod reveal_dr;
mod timeout_actions;

#[test]
fn check_data_request_id() {
    // Expected DR ID for following DR:
    // {
    //     "version": "0.0.1",
    //     "exec_program_id": "044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
    //     "exec_inputs": "ZHJfaW5wdXRz",
    //     "exec_gas_limit": 1,
    //     "tally_program_id": "3a1561a3d854e446801b339c137f87dbd2238f481449c00d3470cfcc2a4e24a1",
    //     "tally_inputs": "dGFsbHlfaW5wdXRz",
    //     "tally_gas_limit": 1,
    //     "replication_factor": 1,
    //     "consensus_filter": "AA==",
    //     "gas_price": 10,
    //     "memo": "XTtTqpLgvyGr54/+ov83JyG852lp7VqzBrC10UpsIjg="
    //   }
    let expected_dr_id = "2404059f879876ad51abe32ad9099d5fe4085c473d54571f109d637a25d62885";

    // compute and check if dr id matches expected value
    let dr = test_helpers::calculate_dr_id_and_args(0, 1);
    let dr_id = dr.try_hash().unwrap();
    assert_eq!(hex::encode(dr_id), expected_dr_id);
}
