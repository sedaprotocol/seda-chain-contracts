use cosmwasm_std::Storage;
use cw_storage_plus::Bound;

use super::*;
mod data_requests_map;
pub use data_requests_map::DataRequestsMap;
mod enumerable_map;
pub use enumerable_map::EnumerableSet;

#[cfg(test)]
mod types_tests;

#[macro_export]
macro_rules! enumerable_status_map {
    ($namespace:literal) => {
        DataRequestsMap {
            reqs:       Map::new(concat!($namespace, "_reqs")),
            committing: $crate::enumerable_set!(concat!($namespace, "_committing")),
            revealing:  $crate::enumerable_set!(concat!($namespace, "_revealing")),
            tallying:   $crate::enumerable_set!(concat!($namespace, "_tallying")),
        }
    };
}

#[macro_export]
macro_rules! enumerable_set {
    ($namespace:expr) => {
        EnumerableSet {
            len:          Item::new(concat!($namespace, "_len")),
            key_to_index: Map::new(concat!($namespace, "_key_to_index")),
            index_to_key: Map::new(concat!($namespace, "_index_to_key")),
        }
    };
}
