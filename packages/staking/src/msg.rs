use crate::state::Config;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum StakingSudoMsg {
    SetConfig { config: Config },
}
