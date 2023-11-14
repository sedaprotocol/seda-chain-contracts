use cosmwasm_std::{QuerierWrapper, StdResult};

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

        let res: QuerySeedResponse = self.querier.query(&request.into())?;
        Ok(res)
    }
}

