#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{BinaryStruct, BINARIES, BINARIES_COUNT},
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

    BINARIES_COUNT.save(deps.storage, &0)?;

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
        binary,
        description,
    };
    // save the binary with a key of the current binaries count
    let key = BINARIES_COUNT.load(deps.storage)?;
    BINARIES.save(deps.storage, &key, &binary_struct)?;

    // increment the binaries count
    BINARIES_COUNT.update(
        deps.storage,
        |mut new_binaries_count| -> Result<_, ContractError> {
            new_binaries_count += 1;
            Ok(new_binaries_count)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "store_binary")
        .add_attribute("new_binary_key", key.to_string()))
}

/// Deletes a binary (without any checks)
pub fn delete_binary(
    deps: DepsMut,
    _info: MessageInfo,
    key: &u128,
) -> Result<Response, ContractError> {
    BINARIES.remove(deps.storage, key);

    Ok(Response::new()
        .add_attribute("method", "delete_binary")
        .add_attribute("deleted_binary_key", key.to_string()))
}

/// Queries a binary and its description by key
pub fn query_binary(deps: Deps, key: &u128) -> StdResult<BinaryStruct> {
    let binary = BINARIES.load(deps.storage, key)?;
    Ok(binary)
}
