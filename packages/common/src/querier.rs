use cosmwasm_std::{
    from_json, to_json_vec, ContractResult, QuerierWrapper, StdError, StdResult, SystemResult,
};

use crate::msg::{QuerySeedResponse, SpecialQueryWrapper};

pub struct SpecialQuerier<'a> {
    querier: &'a QuerierWrapper<'a, SpecialQueryWrapper>,
}

impl<'a> SpecialQuerier<'a> {
    pub fn new(querier: &'a QuerierWrapper<'a, SpecialQueryWrapper>) -> Self {
        SpecialQuerier { querier }
    }

    pub fn query_seed(&self) -> StdResult<QuerySeedResponse> {
        let request = SpecialQueryWrapper {
            query_data: crate::msg::SpecialQueryMsg::QuerySeedRequest {},
        };

        // let res: QuerySeedResponse = self.querier.query(&request.into())?;

        let req_vec = to_json_vec(&request)?;

        match self.querier.raw_query(&req_vec) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {system_err}"
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {contract_err}"),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => Ok(from_json(value)?),
        }

        // Ok(res)
    }
}