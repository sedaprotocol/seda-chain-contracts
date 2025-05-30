use std::num::NonZero;

use cosmwasm_std::Uint128;

pub const INITIAL_MINIMUM_STAKE: Uint128 = Uint128::new(10_000_000_000_000_000_000_000);

pub const INITIAL_COMMIT_TIMEOUT_IN_BLOCKS: NonZero<u8> = NonZero::new(50).unwrap();
pub const INITIAL_REVEAL_TIMEOUT_IN_BLOCKS: NonZero<u8> = NonZero::new(5).unwrap();
pub const INITIAL_BACKUP_DELAY_IN_BLOCKS: NonZero<u8> = NonZero::new(2).unwrap();
// 24 KB
pub const INITIAL_DR_REVEAL_SIZE_LIMIT_IN_BYTES: NonZero<u16> = NonZero::new(24_000).unwrap();
// 2 KB
pub const INITIAL_EXEC_INPUT_LIMIT_IN_BYTES: NonZero<u16> = NonZero::new(2_048).unwrap();
// 512 B
pub const INITIAL_TALLY_INPUT_LIMIT_IN_BYTES: NonZero<u16> = NonZero::new(512).unwrap();
// 512 B
pub const INITIAL_CONSENSUS_FILTER_LIMIT_IN_BYTES: NonZero<u16> = NonZero::new(512).unwrap();
// 512 B
pub const INITIAL_MEMO_LIMIT_IN_BYTES: NonZero<u16> = NonZero::new(512).unwrap();
