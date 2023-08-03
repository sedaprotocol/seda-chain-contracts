#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, DepsMut, MessageInfo, Response, Env, Deps, StdResult};
use cw2::set_contract_version;

use crate::{error::ContractError, state::{BinaryStruct, BINARIES, Config, CONFIG}, msg::{InstantiateMsg, ExecuteMsg, QueryMsg}};

// version info
const CONTRACT_NAME: &str = "seda-bin-storage";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
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
            key,
            binary,
            description,
        } => store_binary(deps, info, &key, binary, description),
        ExecuteMsg::DeleteEntry { key } => delete_binary(deps, info, &key),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> cosmwasm_std::StdResult<Binary> {
    match msg {
        QueryMsg::QueryEntry { key } => cosmwasm_std::to_binary(&query_binary(deps, &key)?),
    }
}

// Should we add the owner to the binary and only the can delete it?
// Or even a list of owners.
pub fn delete_binary(
    deps: DepsMut,
    _info: MessageInfo,
    key: &str,
) -> Result<Response, ContractError> {
    BINARIES.remove(deps.storage, &key);

    Ok(Response::new()
        .add_attribute("method", "delete_binary")
        .add_attribute("deleted_binary_key", key))
}

// Anyone can query it.
pub fn query_binary(deps: Deps, key: &str) -> StdResult<BinaryStruct> {
    let binary = BINARIES.load(deps.storage, key)?;
    Ok(binary)
}

// Awkward this function is the same as above. It's fine it's just to see if this concept works.
pub fn read_binary(
    deps: &DepsMut,
    key: &str,
) -> Result<BinaryStruct, ContractError> {
    let binary_struct = BINARIES.load(deps.storage, key)?;
    Ok(binary_struct)
}

// Should only we be able to store non wasm binaries??
pub fn store_binary(
    deps: DepsMut,
    _info: MessageInfo,
    key: &str,
    binary: Binary,
    description: String,
) -> Result<Response, ContractError> {
    let binary_struct = BinaryStruct {
        binary,
        description,
    };

    if read_binary(&deps, key).is_err() {
        BINARIES.save(deps.storage, key, &binary_struct)?;
        
        Ok(Response::new()
        .add_attribute("method", "store_binary")
        .add_attribute("new_binary_key", key))
    } else {
        Err(ContractError::Conflict(key.to_string()))
    }
}
