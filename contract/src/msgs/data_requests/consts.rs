use cosmwasm_std::Uint128;

const TERA_GAS: u64 = 1_000_000_000_000;

pub const MIN_GAS_PRICE: Uint128 = Uint128::new(2_000);
pub const MIN_EXEC_GAS_LIMIT: u64 = 10 * TERA_GAS;
pub const MIN_TALLY_GAS_LIMIT: u64 = 10 * TERA_GAS;

pub const MAX_REPLICATION_FACTOR: u16 = 100;

#[cfg(test)]
pub fn min_post_dr_cost() -> u128 {
    let exec_gas_limit = Uint128::new(MIN_EXEC_GAS_LIMIT as u128);
    let tally_gas_limit = Uint128::new(MIN_TALLY_GAS_LIMIT as u128);

    ((exec_gas_limit + tally_gas_limit) * MIN_GAS_PRICE).u128()
}
