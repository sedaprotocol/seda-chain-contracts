use std::num::NonZero;

use cosmwasm_std::Uint128;

pub const INITIAL_MINIMUM_STAKE: Uint128 = Uint128::new(10_000_000_000_000_000_000_000);

pub const INITIAL_COMMIT_TIMEOUT_IN_BLOCKS: u64 = 50;
pub const INITIAL_REVEAL_TIMEOUT_IN_BLOCKS: u64 = 5;
pub const INITIAL_BACKUP_DELAY_IN_BLOCKS: NonZero<u64> = NonZero::new(2).unwrap();
