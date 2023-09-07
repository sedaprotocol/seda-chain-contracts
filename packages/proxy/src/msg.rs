use common::msg::PostDataRequestArgs;
use common::state::Reveal;
use common::types::Hash;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Admin
    SetSedaChainContracts { contract: String },

    // Delegated calls to contracts
    PostDataRequest { posted_dr: PostDataRequestArgs },
    CommitDataResult { dr_id: Hash, commitment: String },
    RevealDataResult { dr_id: Hash, reveal: Reveal },
    RegisterDataRequestExecutor { p2p_multi_address: Option<String> },
    UnregisterDataRequestExecutor {},
    DepositAndStake,
    Unstake { amount: u128 },
    Withdraw { amount: u128 },
}
