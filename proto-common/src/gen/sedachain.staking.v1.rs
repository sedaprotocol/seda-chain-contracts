// @generated
// This file is @generated by prost-build.
/// MsgCreateSEDAValidator defines a message for creating a new SEDA
/// validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateSedaValidator {
    #[prost(message, optional, tag="1")]
    pub description: ::core::option::Option<::cosmos_sdk_proto::cosmos::staking::v1beta1::Description>,
    #[prost(message, optional, tag="2")]
    pub commission: ::core::option::Option<::cosmos_sdk_proto::cosmos::staking::v1beta1::CommissionRates>,
    #[prost(string, tag="3")]
    pub min_self_delegation: ::prost::alloc::string::String,
    /// Deprecated: Use of Delegator Address in MsgCreateValidator is deprecated.
    /// The validator address bytes and delegator address bytes refer to the same
    /// account while creating validator (defer only in bech32 notation).
    #[deprecated]
    #[prost(string, tag="4")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag="6")]
    pub pubkey: ::core::option::Option<::prost_types::Any>,
    #[prost(message, optional, tag="7")]
    pub value: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag="8")]
    pub indexed_pub_keys: ::prost::alloc::vec::Vec<super::pubkey::IndexedPubKey>,
}
impl ::prost::Name for MsgCreateSedaValidator {
const NAME: &'static str = "MsgCreateSEDAValidator";
const PACKAGE: &'static str = "sedachain.staking.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.staking.v1.MsgCreateSEDAValidator".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.staking.v1.MsgCreateSEDAValidator".into() }}
/// MsgCreateSEDAValidatorResponse defines the Msg/MsgCreateSEDAValidator
/// response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct MsgCreateSedaValidatorResponse {
}
impl ::prost::Name for MsgCreateSedaValidatorResponse {
const NAME: &'static str = "MsgCreateSEDAValidatorResponse";
const PACKAGE: &'static str = "sedachain.staking.v1";
fn full_name() -> ::prost::alloc::string::String { "sedachain.staking.v1.MsgCreateSEDAValidatorResponse".into() }fn type_url() -> ::prost::alloc::string::String { "/sedachain.staking.v1.MsgCreateSEDAValidatorResponse".into() }}
include!("sedachain.staking.v1.tonic.rs");
// @@protoc_insertion_point(module)