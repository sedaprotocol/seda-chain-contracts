use super::*;

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(feature = "cosmwasm", derive(QueryResponses))]
#[cfg_attr(not(feature = "cosmwasm"), derive(Serialize))]
#[cfg_attr(not(feature = "cosmwasm"), serde(rename_all = "snake_case"))]
pub enum QueryMsg {
    #[cfg_attr(feature = "cosmwasm", returns(DataRequest))]
    GetDataRequest { dr_id: Hash },
    #[cfg_attr(feature = "cosmwasm",  returns(Option<Hash>))]
    GetDataRequestCommitment { dr_id: Hash, public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm",  returns(HashMap<String, Hash>))]
    GetDataRequestCommitments { dr_id: Hash },
    #[cfg_attr(feature = "cosmwasm",  returns(Option<RevealBody>))]
    GetDataRequestReveal { dr_id: Hash, public_key: PublicKey },
    #[cfg_attr(feature = "cosmwasm",  returns(HashMap<String, RevealBody>))]
    GetDataRequestReveals { dr_id: Hash },
    #[cfg_attr(feature = "cosmwasm", returns(DataResult))]
    GetResolvedDataRequest { dr_id: Hash },
    #[cfg_attr(feature = "cosmwasm",  returns(HashMap<String, DR>))]
    GetDataRequestsByStatus { status: DataRequestStatus },
}

impl From<QueryMsg> for super::QueryMsg {
    fn from(value: QueryMsg) -> Self {
        Self::DataRequest(value)
    }
}
