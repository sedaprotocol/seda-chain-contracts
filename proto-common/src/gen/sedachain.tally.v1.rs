// @generated
// This file is @generated by prost-build.
/// Params defines the parameters for the tally module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// MaxTallyGasLimit is the maximum gas limit for a tally request.
    #[prost(uint64, tag="1")]
    pub max_tally_gas_limit: u64,
    /// FilterGasCostNone is the gas cost for a filter type none.
    #[prost(uint64, tag="2")]
    pub filter_gas_cost_none: u64,
    /// FilterGasCostMultiplierMode is the gas cost multiplier for a filter type
    /// mode.
    #[prost(uint64, tag="3")]
    pub filter_gas_cost_multiplier_mode: u64,
    /// FilterGasCostMAD is the gas cost multiplier for a filter type
    /// Median Absolute Deviation.
    #[prost(uint64, tag="4")]
    pub filter_gas_cost_multiplier_m_a_d: u64,
    /// GasCostBase is the base gas cost for a data request.
    #[prost(uint64, tag="5")]
    pub gas_cost_base: u64,
    /// GasCostFallback is the gas cost incurred for data request execution when
    /// even basic consensus has not been reached.
    #[prost(uint64, tag="6")]
    pub execution_gas_cost_fallback: u64,
    /// BurnRatio is the ratio of the gas cost to be burned in case of reduced
    /// payout scenarios.
    #[prost(string, tag="7")]
    pub burn_ratio: ::prost::alloc::string::String,
    /// MaxResultSize is the maximum size of the result of a data request in bytes.
    #[prost(uint32, tag="8")]
    pub max_result_size: u32,
    /// MaxTalliesPerBlock specifies the maximum number of tallies per block.
    #[prost(uint32, tag="9")]
    pub max_tallies_per_block: u32,
}
impl ::prost::Name for Params {
const NAME: &'static str = "Params";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.Params".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.Params".into() }}
/// GenesisState defines tally module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag="1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
const NAME: &'static str = "GenesisState";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.GenesisState".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.GenesisState".into() }}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {
}
impl ::prost::Name for QueryParamsRequest {
const NAME: &'static str = "QueryParamsRequest";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.QueryParamsRequest".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.QueryParamsRequest".into() }}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag="1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
const NAME: &'static str = "QueryParamsResponse";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.QueryParamsResponse".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.QueryParamsResponse".into() }}
/// The request message for the UpdateParams method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag="1")]
    pub authority: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
const NAME: &'static str = "MsgUpdateParams";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.MsgUpdateParams".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.MsgUpdateParams".into() }}
/// The response message for the UpdateParams method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {
}
impl ::prost::Name for MsgUpdateParamsResponse {
const NAME: &'static str = "MsgUpdateParamsResponse";
const PACKAGE: &'static str = "sedachain.tally.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.tally.v1.MsgUpdateParamsResponse".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.tally.v1.MsgUpdateParamsResponse".into() }}
include!("sedachain.tally.v1.tonic.rs");
// @@protoc_insertion_point(module)