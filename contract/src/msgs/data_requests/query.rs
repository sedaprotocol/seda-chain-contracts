use std::ops::Rem;

use cosmwasm_std::Uint256;
use cw_storage_plus::{Bound, IndexedMap};
use sha3::digest::consts::U2;
use staking::state::STAKERS;
use state::may_load_request;

use super::{msgs::data_requests::query::QueryMsg, *};

impl QueryHandler for QueryMsg {
    fn query(self, deps: Deps, env: Env) -> Result<Binary, ContractError> {
        let binary = match self {
            QueryMsg::GetDataRequest { dr_id } => {
                to_json_binary(&state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestCommitment { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.as_ref().map(|dr| dr.get_commitment(&public_key)))?
            }
            QueryMsg::GetDataRequestCommitments { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let commitments = dr.map(|dr| dr.commits).unwrap_or_default();
                to_json_binary(&commitments)?
            }
            QueryMsg::GetDataRequestReveal { dr_id, public_key } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                to_json_binary(&dr.as_ref().map(|dr| dr.get_reveal(&public_key)))?
            }
            QueryMsg::GetDataRequestReveals { dr_id } => {
                let dr = state::may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?;
                let reveals = dr.map(|dr| dr.reveals).unwrap_or_default();
                to_json_binary(&reveals)?
            }
            QueryMsg::GetDataResult { dr_id } => {
                to_json_binary(&state::may_load_result(deps.storage, &Hash::from_hex_str(&dr_id)?)?)?
            }
            QueryMsg::GetDataRequestsByStatus { status, offset, limit } => {
                to_json_binary(&state::requests_by_status(deps.storage, &status, offset, limit)?)?
            },
            QueryMsg::IsEligible { dr_id, public_key } => {
                to_json_binary(&is_eligible(deps, env, dr_id, public_key)?)?
            }
        };

        Ok(binary)
    }
}

fn is_eligible(deps: Deps, env: Env, dr_id: String, public_key: String) -> Result<bool, ContractError> {
    let public_key = PublicKey::from_hex_str(&public_key)?;
    let data_request = may_load_request(deps.storage, &Hash::from_hex_str(&dr_id)?)?.unwrap();

    let stakers = STAKERS.keys(deps.storage, None, None, Order::Ascending);
    let all_stakers = stakers.into_iter().collect::<StdResult<Vec<_>>>()?;
    let stakers_length: u64 = all_stakers.len().try_into().expect("cannot convert to u64");

    let (staker_index, _) = all_stakers.iter().enumerate().find(|(_, &pk)|{
        public_key == pk
    }).expect("Could not find staker");

    let executor_index = Uint256::from(u128::try_from(staker_index).unwrap());

    let dr_id: [u8;32]  = hex::decode(dr_id)?.try_into().unwrap();
    
    let executor_length = Uint256::from(stakers_length);
    let dr_index = Uint256::from_be_bytes(dr_id) % executor_length;
    let replication_factor = Uint256::from(data_request.replication_factor);
    let end_index = (dr_index + replication_factor) % executor_length;
    
    if dr_index < end_index {
        // No overflow case
        Ok(executor_index >= dr_index && executor_index < end_index)
    } else {
        // Overflow case
        Ok(executor_index >= dr_index || executor_index < end_index)
    }

    // if (dr_index + replication_factor - Uint256::one()) > executor_length {
    //     return Ok(executor_index >= dr_index || executor_index <= replication_factor - (executor_length - dr_index) - Uint256::one());
    // } else { //no overflow
    //     return Ok(executor_index >= dr_index && executor_index <= dr_index + replication_factor - Uint256::one());
    // }

    // throws error because substracting from zero
    // this should be a signed integer ok
    // let dw = height_delta - amount_of_blocks_to_wait - 1;
    // let bf: u64 = 0i128.max(2i128 ^ dw).try_into().unwrap();
    // let bf = 0;

    // // low_cond should be 0 right
    // let low_cond = dr_index.min((dr_index - Uint256::from(rf+bf)).rem(stakers_length));
    // let high_cond = stakers_length.min((dr_index + Uint256::from(rf+bf)).rem(stakers_length));
    
    //?
    // I think I know why...
    // println!("dr_index: {}", dr_index);
    // println!("stakers_length: {}", stakers_length);

    // println!("staker_index: {}", staker_index);
    // println!("low_cond: {}", low_cond);
    // println!("high_cond: {}", high_cond);

    // println!("eligible: {}", staker_index >= low_cond && staker_index <= high_cond);
    // println!("--------------");
    
    // Ok(staker_index >= low_cond && staker_index <= high_cond)
}