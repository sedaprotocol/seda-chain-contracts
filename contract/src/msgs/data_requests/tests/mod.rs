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
    //     "exec_program_id":
    // "044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
    //     "exec_inputs": "ZHJfaW5wdXRz",
    //     "exec_gas_limit": 10_000_000_000_000,
    //     "tally_program_id":
    // "e36e73257ac61c4e7126922411591d8558f4e2f6213aca27ef0b1e2ebf05fe35",
    //     "tally_inputs": "dGFsbHlfaW5wdXRz",
    //     "tally_gas_limit": 10_000_000_000_000,
    //     "replication_factor": 1,
    //     "consensus_filter": "AA==",
    //     "gas_price": 2000,
    //     "memo": "XTtTqpLgvyGr54/+ov83JyG852lp7VqzBrC10UpsIjg="
    //   }
    let expected_dr_id = "b5722604a49dc8b83751890f2e85dae02f5c38c5e86a88e262e62f7e9b292de1";

    // compute and check if dr id matches expected value
    let dr = test_helpers::calculate_dr_id_and_args(0, 1);
    let dr_id = dr.try_hash().unwrap();
    assert_eq!(hex::encode(dr_id), expected_dr_id);
}
