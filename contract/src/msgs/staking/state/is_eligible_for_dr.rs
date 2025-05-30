use std::num::NonZero;

use data_requests::state::load_request;
use sha3::{Digest, Keccak256};

use super::{staking::state::STAKERS, *};
use crate::msgs::data_requests::state::DR_CONFIG;

/// Compute deterministic hash for staker selection by combining public key and
/// dr_id
fn compute_selection_hash(public_key: &[u8], dr_id: &[u8]) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(public_key);
    hasher.update(dr_id);
    hasher.finalize().into()
}

/// Check if a staker is eligible to execute a data request
///
/// # Arguments
/// * `deps` - Dependencies for storage access
/// * `env` - Environment info, used for block height
/// * `dr_id` - Unique identifier of the data request
/// * `public_key` - Public key of the staker to check
///
/// # Returns
/// * `Ok(true)` if staker is eligible
/// * `Ok(false)` if staker is not eligible
/// * `Err` if there's an error accessing storage
pub fn is_eligible_for_dr(deps: Deps, env: Env, dr_id: [u8; 32], public_key: PublicKey) -> Result<bool, ContractError> {
    let data_request = load_request(deps.storage, &dr_id)?;
    let config = STAKING_CONFIG.load(deps.storage)?;

    if !STAKERS.is_staker_executor(deps.storage, &public_key)? {
        return Ok(false);
    }

    let stakers = STAKERS
        .stakers
        .range_raw(deps.storage, None, None, Order::Ascending)
        .flatten();
    let blocks_passed = env.block.height - data_request.base.height;
    let dr_config = DR_CONFIG.load(deps.storage)?;

    Ok(calculate_dr_eligibility(
        stakers,
        public_key.as_ref(),
        config.minimum_stake,
        dr_config.backup_delay_in_blocks,
        dr_id,
        data_request.base.replication_factor,
        blocks_passed,
    ))
}

/// Calculate if a staker is eligible based on their position in a deterministic
/// ordering
///
/// # Arguments
/// * `active_stakers` - Iterator of (public_key, staker) pairs
/// * `target_public_key` - Public key to check eligibility for
/// * `minimum_stake` - Minimum stake required to be considered active
/// * `dr_id` - Data request ID used for deterministic selection
/// * `replication_factor` - Initial number of nodes needed
/// * `blocks_passed` - Number of blocks since DR was posted
///
/// # Algorithm
/// 1. Filter stakers that meet minimum stake requirement
/// 2. Compute hash for target staker using public_key + dr_id
/// 3. Count total eligible stakers and how many have lower hash than target
/// 4. Calculate total needed stakers (replication_factor + blocks_passed),
///    capped by total available
/// 5. Target is eligible if fewer stakers have lower hash than total needed
///
/// # Returns
/// `true` if the staker is eligible, `false` otherwise
fn calculate_dr_eligibility<I>(
    active_stakers: I,
    target_public_key: &[u8],
    minimum_stake: Uint128,
    backup_delay_in_blocks: NonZero<u8>,
    dr_id: [u8; 32],
    replication_factor: u16,
    blocks_passed: u64,
) -> bool
where
    I: Iterator<Item = (Vec<u8>, Staker)>,
{
    let target_hash = compute_selection_hash(target_public_key, &dr_id);

    // Count total eligible stakers and stakers with lower hash in one pass
    let (total_stakers, lower_hash_count) = active_stakers
        .filter(|(_, staker)| staker.tokens_staked >= minimum_stake)
        .fold((0, 0), |(total, lower), (public_key, _)| {
            let staker_hash = compute_selection_hash(&public_key, &dr_id);
            (total + 1, lower + if staker_hash < target_hash { 1 } else { 0 })
        });

    if total_stakers == 0 {
        return false;
    }

    // Calculate total needed stakers, capped by total available
    // for someone to be eligible after the first executor the number of blocks
    // passed needs to be greater than the backup delay. After the delay a new
    // staker is eligible every block.
    let total_needed = if blocks_passed > backup_delay_in_blocks.get() as u64 {
        replication_factor as u64 + (blocks_passed - backup_delay_in_blocks.get() as u64)
    } else {
        replication_factor as u64
    };
    let total_needed = total_needed.min(total_stakers as u64);

    // Staker is eligible if their position (by hash order) is within needed range
    lower_hash_count < total_needed as usize
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_stakers(count: usize) -> Vec<(Vec<u8>, Staker)> {
        (1..=count)
            .map(|i| {
                (
                    vec![i as u8],
                    Staker {
                        tokens_staked:             Uint128::from(100u128),
                        memo:                      None,
                        tokens_pending_withdrawal: Uint128::from(0u128),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn test_node_selection_consistency() {
        let dr_id = [1; 32];
        let minimum_stake = Uint128::from(100u128);
        let backup_delay_in_blocks = NonZero::new(2).unwrap();
        let replication_factor = 3;
        let blocks_passed = 2;
        let stakers = create_test_stakers(10);

        // Get selected nodes using sorting approach
        let mut sorted_stakers = stakers.clone();
        sorted_stakers.sort_by_cached_key(|(public_key, _)| compute_selection_hash(public_key, &dr_id));

        // let total_needed = (replication_factor as usize + blocks_passed as
        // usize).min(sorted_stakers.len());
        let total_needed = if blocks_passed > backup_delay_in_blocks.get() as u64 {
            replication_factor as usize + (blocks_passed - backup_delay_in_blocks.get() as u64) as usize
        } else {
            replication_factor as usize
        }
        .min(sorted_stakers.len());
        let selected_by_sorting: Vec<_> = sorted_stakers
            .iter()
            .take(total_needed)
            .map(|(pk, _)| pk.clone())
            .collect();

        // Get selected nodes using eligibility function
        let selected_by_eligibility: Vec<_> = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    blocks_passed,
                )
            })
            .map(|(pk, _)| pk.clone())
            .collect();

        // Sort both results for comparison
        let mut sorted_by_sorting = selected_by_sorting.clone();
        let mut sorted_by_eligibility = selected_by_eligibility.clone();
        sorted_by_sorting.sort();
        sorted_by_eligibility.sort();

        // Verify both approaches select the same nodes
        assert_eq!(
            sorted_by_sorting, sorted_by_eligibility,
            "Selected nodes don't match between sorting and eligibility approaches"
        );

        // Verify we selected the correct number of nodes
        assert_eq!(
            selected_by_eligibility.len(),
            total_needed,
            "Wrong number of nodes selected"
        );

        // Test with different dr_id to ensure selection changes
        let different_dr_id = [2; 32];
        let selected_with_different_dr: Vec<_> = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    different_dr_id,
                    replication_factor,
                    blocks_passed,
                )
            })
            .map(|(pk, _)| pk.clone())
            .collect();

        assert_ne!(
            selected_by_eligibility, selected_with_different_dr,
            "Node selection should change with different dr_id"
        );
    }

    #[test]
    fn test_backup_replication_factor() {
        let minimum_stake = Uint128::from(100u128);
        let backup_delay_in_blocks = NonZero::new(2).unwrap();
        let dr_id = [1; 32];
        let replication_factor = 2;
        let stakers = create_test_stakers(5);

        // Test with 0 blocks passed (should use replication_factor)
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    0,
                )
            })
            .count();
        assert_eq!(eligible_count, replication_factor as usize);

        // Test with 4 blocks passed (should use replication_factor + 2)
        // since delay is 2 and 4 blocks passed
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    4,
                )
            })
            .count();
        assert_eq!(eligible_count, (replication_factor + 2) as usize);

        // Test with many blocks passed (should cap at total stakers)
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    10,
                )
            })
            .count();
        assert_eq!(eligible_count, stakers.len());
    }

    #[test]
    fn test_basic_backup_delay() {
        let minimum_stake = Uint128::from(100u128);
        let backup_delay_in_blocks = NonZero::new(1).unwrap();
        let dr_id = [1; 32];
        let replication_factor = 1;
        let stakers = create_test_stakers(5);

        // post on block 10
        // a new executor should be available on block 12 and after
        // with a backup delay of 1

        // Test with 1 block passed (should not use backup delay)
        // after block 11 and before only the asked for replication factor should be
        // available
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    1,
                )
            })
            .count();
        assert_eq!(eligible_count, replication_factor as usize);

        // Test with 2 blocks passed (should use backup delay)
        // i.e. after block 12 and up one more executor should be available per block
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    2,
                )
            })
            .count();
        assert_eq!(eligible_count, (replication_factor + 1) as usize);

        // Test with 3 blocks passed (should use backup delay)
        // block 13 posted so another executor should be available
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    3,
                )
            })
            .count();
        assert_eq!(eligible_count, (replication_factor + 2) as usize);
    }

    #[test]
    fn test_larger_backup_delay() {
        let minimum_stake = Uint128::from(100u128);
        let backup_delay_in_blocks = NonZero::new(5).unwrap();
        let dr_id = [1; 32];
        let replication_factor = 2;
        let stakers = create_test_stakers(5);

        // say we post on block 10
        // Test with 4 blocks passed (should not use backup delay)
        // i.e on block 15 and before only the asked for replication factor should be
        // available
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    4,
                )
            })
            .count();
        assert_eq!(eligible_count, replication_factor as usize);

        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    5,
                )
            })
            .count();
        assert_eq!(eligible_count, replication_factor as usize);

        // Test with 6 blocks passed (should use backup delay)
        // i.e. after block 16 posted and up one more executor should be available
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    6,
                )
            })
            .count();
        assert_eq!(eligible_count, (replication_factor + 1) as usize);

        // Test with 7 blocks passed (should use backup delay)
        // another block passed so another executor should be available
        let eligible_count = stakers
            .iter()
            .filter(|(public_key, _)| {
                calculate_dr_eligibility(
                    stakers.clone().into_iter(),
                    public_key,
                    minimum_stake,
                    backup_delay_in_blocks,
                    dr_id,
                    replication_factor,
                    7,
                )
            })
            .count();
        assert_eq!(eligible_count, (replication_factor + 2) as usize);
    }
}
