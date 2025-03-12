#[cfg(feature = "batching")]
#[path = "sedachain.batching.v1.rs"]
#[rustfmt::skip]
pub mod batching;

#[cfg(feature = "data_proxy")]
#[path = "sedachain.data_proxy.v1.rs"]
#[rustfmt::skip]
pub mod data_proxy;

#[cfg(feature = "pubkey")]
#[path = "sedachain.pubkey.v1.rs"]
#[rustfmt::skip]
pub mod pubkey;

#[cfg(feature = "randomness")]
#[path = "sedachain.randomness.v1.rs"]
#[rustfmt::skip]
pub mod randomness;

#[cfg(feature = "staking")]
#[path = "sedachain.staking.v1.rs"]
#[rustfmt::skip]
pub mod staking;

#[cfg(feature = "vesting")]
#[path = "sedachain.vesting.v1.rs"]
#[rustfmt::skip]
pub mod vesting;

#[cfg(feature = "wasm_storage")]
#[path = "sedachain.wasm_storage.v1.rs"]
#[rustfmt::skip]
pub mod wasm_storage;

// re-export prost
#[cfg(feature = "cosmos")]
pub use cosmos_sdk_proto;
pub use prost;
