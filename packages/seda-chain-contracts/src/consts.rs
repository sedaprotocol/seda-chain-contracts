// TODO: these should be governance-controlled parameters in state
pub const MINIMUM_STAKE_TO_REGISTER: u128 = 1;

// A set amount that marks data request executor as eligible for committee inclusion.
pub const MINIMUM_STAKE_FOR_COMMITTEE_ELIGIBILITY: u128 = 1;

// a threshold after which revealing starts
pub const COMMITS_THRESHOLD: u128 = 3;

// a threshold after which tally starts
pub const REVEAL_THRESHOLD: u128 = 2;