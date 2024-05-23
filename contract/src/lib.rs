pub mod config;
pub mod consts;
pub mod contract;
mod crypto;
mod error;
pub mod msg;
pub mod msgs;
pub mod staking;
pub mod state;
mod types;
mod utils;

// #[path =""]
// #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
// mod lib {
//  pub mod consts;
// }

#[path = ""]
#[cfg(test)]
pub(crate) mod test {
    pub mod test_helpers;
    pub mod test_utils;

    mod config_test;
    mod staking_test;
}
