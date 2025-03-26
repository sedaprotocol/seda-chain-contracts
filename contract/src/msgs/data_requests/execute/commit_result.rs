use staking::state::{STAKERS, STAKING_CONFIG};

use super::*;
use crate::{msgs::owner::state::ALLOWLIST, state::CHAIN_ID};

impl ExecuteHandler for execute::commit_result::Execute {
    /// Posts a data result of a data request with an attached hash of the answer and salt.
    fn execute(self, deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
        // find the data request from the pool (if it exists, otherwise error)
        let dr_id = Hash::from_hex_str(&self.dr_id)?;
        let mut dr = state::load_request(deps.storage, &dr_id)?;

        verify_commit(deps.as_ref(), &env, &self, &dr)?;

        // add the commitment to the data request
        let commitment = Hash::from_hex_str(&self.commitment)?;
        dr.commits.insert(self.public_key.clone(), commitment);

        let resp = Response::new().add_attribute("action", "commit_data_result").add_event(
            Event::new("seda-commitment").add_attributes([
                ("dr_id", self.dr_id.clone()),
                ("posted_dr_height", dr.height.to_string()),
                ("commitment", self.commitment),
                ("executor", self.public_key.clone()),
                ("version", CONTRACT_VERSION.to_string()),
            ]),
        );
        state::commit(deps.storage, env.block.height, &dr_id, dr)?;

        Ok(resp.add_message(new_refund_msg(env, self.dr_id, self.public_key, false)?))
    }
}

pub fn verify_commit(
    deps: Deps,
    env: &Env,
    commit: &execute::commit_result::Execute,
    dr: &DataRequest,
) -> Result<(), ContractError> {
    // error if the user has already committed
    if dr.has_committer(commit.public_key.as_str()) {
        return Err(ContractError::AlreadyCommitted);
    }

    // error if reveal stage has started (replication factor reached)
    if dr.reveal_started() {
        return Err(ContractError::RevealStarted);
    }

    // error if the data request has expired
    let expires_at = state::get_dr_expiration_height(deps.storage, &Hash::from_hex_str(&dr.id)?)?;
    if expires_at <= env.block.height {
        return Err(ContractError::DataRequestExpired(expires_at, "commit"));
    }

    let public_key = PublicKey::from_hex_str(commit.public_key.as_str())?;
    let staker = STAKERS.get_staker(deps.storage, &public_key)?;

    // Check if the staker is on the allowlist if it is enabled
    if STAKING_CONFIG.load(deps.storage)?.allowlist_enabled {
        let allowed = ALLOWLIST.may_load(deps.storage, &public_key)?;
        if allowed.is_none() || !allowed.unwrap() {
            return Err(ContractError::NotOnAllowlist);
        }
    }

    // Check if the staker has enough funds staked to commit
    let minimum_stake = STAKING_CONFIG.load(deps.storage)?.minimum_stake;
    if staker.tokens_staked < minimum_stake {
        return Err(ContractError::InsufficientFunds(minimum_stake, staker.tokens_staked));
    }

    // verify the proof
    let chain_id = CHAIN_ID.load(deps.storage)?;
    commit.verify(public_key.as_ref(), &chain_id, env.contract.address.as_str(), dr.height)?;
    Ok(())
}
