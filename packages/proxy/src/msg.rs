use cosmwasm_schema::{cw_serde, QueryResponses};
use seda_chain_contracts::msg::PostDataRequestArgs;
use seda_chain_contracts::state::Reveal;
use seda_chain_contracts::types::Hash;

#[cw_serde]
pub struct InstantiateMsg {}

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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // #[returns(crate::state::BinaryStruct)]
    // QueryEntry { key: String },
}
