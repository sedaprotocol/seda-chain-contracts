use common::state::StakingConfig;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum StakingSudoMsg {
    SetStakingConfig { config: StakingConfig },
}
