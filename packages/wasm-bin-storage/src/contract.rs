#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use sha3::{Digest, Keccak256};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{BinaryStruct, BINARIES},
};

// version info
const CONTRACT_NAME: &str = "seda-bin-storage";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::NewEntry {
            binary,
            description,
        } => store_binary(deps, info, binary, description),
        ExecuteMsg::DeleteEntry { key } => delete_binary(deps, info, &key),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> cosmwasm_std::StdResult<Binary> {
    match msg {
        QueryMsg::QueryEntry { key } => cosmwasm_std::to_binary(&query_binary(deps, &key)?),
    }
}

/// Stores a binary along with a description and returns a key to retrieve it
pub fn store_binary(
    deps: DepsMut,
    _info: MessageInfo,
    binary: Vec<u8>,
    description: String,
) -> Result<Response, ContractError> {
    let binary_struct = BinaryStruct {
        binary: binary.clone(),
        description,
    };

    // calculate the hash of the binary
    let mut hasher = Keccak256::new();
    hasher.update(&binary);
    let binary_hash = format!("0x{}", hex::encode(hasher.finalize()));

    // require hash to be unique
    if BINARIES.may_load(deps.storage, &binary_hash)?.is_some() {
        return Err(ContractError::BinaryAlreadyExists {});
    }

    // save the binary
    BINARIES.save(deps.storage, &binary_hash, &binary_struct)?;

    Ok(Response::new()
        .add_attribute("method", "store_binary")
        .add_attribute("new_binary_key", binary_hash))
}

/// Deletes a binary (without any checks)
pub fn delete_binary(
    deps: DepsMut,
    _info: MessageInfo,
    key: &String,
) -> Result<Response, ContractError> {
    BINARIES.remove(deps.storage, key);

    Ok(Response::new()
        .add_attribute("method", "delete_binary")
        .add_attribute("deleted_binary_key", key))
}

/// Queries a binary and its description by key
pub fn query_binary(deps: Deps, key: &String) -> StdResult<BinaryStruct> {
    let binary = BINARIES.load(deps.storage, key)?;
    Ok(binary)
}
