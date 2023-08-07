use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};
use sha3::{Digest, Keccak256};

use crate::error::ContractError;
use crate::msg::ExecuteMsg;

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn get_attached_funds(funds: &[Coin], token: String) -> Result<u128, ContractError> {
    let amount: Option<u128> = funds
        .iter()
        .find(|coin| coin.denom == token)
        .map(|coin| coin.amount.u128());
    amount.ok_or(ContractError::NoFunds)
}

pub fn pad_to_32_bytes(value: u128) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let small_bytes = &value.to_be_bytes();
    bytes[(32 - small_bytes.len())..].copy_from_slice(small_bytes);
    bytes
}

pub fn hash_update(hasher: &mut Keccak256, value: u128) {
    let bytes = pad_to_32_bytes(value);
    hasher.update(bytes);
}
