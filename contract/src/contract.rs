#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cosmwasm_std::{Empty, Event};
use cw2::{get_contract_version, set_contract_version};
use data_requests::DrConfig;
use seda_common::msgs::*;
use semver::Version;
use staking::StakingConfig;

use crate::{
    consts::*,
    error::ContractError,
    msgs::{
        data_requests::{execute::dr_events::create_dr_config_event, state::DR_CONFIG},
        owner::state::{OWNER, PENDING_OWNER},
        staking::{
            execute::staking_events::create_staking_config_event,
            state::{STAKERS, STAKING_CONFIG},
        },
        ExecuteHandler,
        QueryHandler,
        SudoHandler,
    },
    state::{CHAIN_ID, PAUSED, TOKEN},
};

// version info for migration info
const CONTRACT_NAME: &str = "seda-core-contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_REVISION: &str = env!("GIT_REVISION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    #[cfg(not(test))]
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    #[cfg(test)]
    {
        let version = std::env::var("TEST_CONTRACT_VERSION").unwrap_or_else(|_| "1.0.0".to_string());

        set_contract_version(deps.storage, CONTRACT_NAME, &version)?;
    }
    TOKEN.save(deps.storage, &msg.token)?;
    OWNER.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    CHAIN_ID.save(deps.storage, &msg.chain_id)?;
    PENDING_OWNER.save(deps.storage, &None)?;
    PAUSED.save(deps.storage, &false)?;

    let init_staking_config = msg.staking_config.unwrap_or(StakingConfig {
        minimum_stake:     INITIAL_MINIMUM_STAKE,
        allowlist_enabled: true,
    });

    if init_staking_config.minimum_stake.is_zero() {
        return Err(ContractError::ZeroMinimumStakeToRegister);
    }

    STAKING_CONFIG.save(deps.storage, &init_staking_config)?;

    let init_dr_config = msg.dr_config.unwrap_or(DrConfig {
        commit_timeout_in_blocks: INITIAL_COMMIT_TIMEOUT_IN_BLOCKS,
        reveal_timeout_in_blocks: INITIAL_REVEAL_TIMEOUT_IN_BLOCKS,
        backup_delay_in_blocks:   INITIAL_BACKUP_DELAY_IN_BLOCKS,
    });
    DR_CONFIG.save(deps.storage, &init_dr_config)?;

    STAKERS.initialize(deps.storage)?;
    crate::msgs::data_requests::state::init_data_requests(deps.storage)?;

    Ok(Response::new().add_attribute("method", "instantiate").add_events([
        Event::new("seda-contract").add_attributes([
            ("action", "instantiate".to_string()),
            ("version", CONTRACT_VERSION.to_string()),
            ("chain_id", msg.chain_id),
            ("owner", msg.owner),
            ("token", msg.token),
            ("git_revision", GIT_REVISION.to_string()),
        ]),
        create_staking_config_event(init_staking_config),
        create_dr_config_event(init_dr_config),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    msg.execute(deps, env, info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, sudo: SudoMsg) -> Result<Response, ContractError> {
    sudo.sudo(deps, env)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    msg.query(deps, env)
}

/// Migrate the contract to a new version, emitting an event with the migration
/// details.
///
/// # Errors
///
/// Returns [`DowngradeNotSupported`](ContractError::DowngradeNotSupported) if
/// trying to downgrade the contract.
///
/// Returns [`NoMigrationNeeded`](ContractError::NoMigrationNeeded) if the
/// contract is already at the latest version.
///
/// Returns [`SemVer`](ContractError::NoMigrationNeeded) if the new or old
/// version is not semver compatible.
///
/// Returns [`Std`](ContractError::Std) if the migration fails. Getting/setting
/// the contract version. Or loading the chain ID from storage.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version > version {
        return Err(ContractError::DowngradeNotSupported);
    }

    if storage_version == version {
        return Err(ContractError::NoMigrationNeeded);
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_event(Event::new("seda-contract").add_attributes([
            ("action", "migrate".to_string()),
            ("current_version", storage_version.to_string()),
            ("target_version", version.to_string()),
            ("chain_id", CHAIN_ID.load(deps.storage)?.to_string()),
            ("git_revision", GIT_REVISION.to_string()),
        ])))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use cw_multi_test::{ContractWrapper, Executor};

    use super::*;
    use crate::TestInfo;

    #[test]
    fn migrate_downgrade() {
        let test_info = TestInfo::init_with_version(Some("2.0.0"));

        let contract = Box::new(
            ContractWrapper::new(execute, instantiate, query)
                .with_sudo(sudo)
                .with_migrate_empty(migrate),
        );

        let new_code_id = test_info
            .app_mut()
            .store_code_with_creator(test_info.creator().addr(), contract);

        assert!(test_info
            .app_mut()
            .migrate_contract(
                test_info.creator().addr(),
                test_info.contract_addr(),
                &Empty {},
                new_code_id,
            )
            .unwrap_err()
            .source()
            .unwrap()
            .to_string()
            .contains("Cannot downgrade contract version"));
    }

    #[test]
    fn migrate_no_upgrade() {
        let test_info = TestInfo::init_with_version(Some(CONTRACT_VERSION));

        let contract = Box::new(
            ContractWrapper::new(execute, instantiate, query)
                .with_sudo(sudo)
                .with_migrate_empty(migrate),
        );

        let new_code_id = test_info
            .app_mut()
            .store_code_with_creator(test_info.creator().addr(), contract);

        assert!(test_info
            .app_mut()
            .migrate_contract(
                test_info.creator().addr(),
                test_info.contract_addr(),
                &Empty {},
                new_code_id,
            )
            .unwrap_err()
            .source()
            .unwrap()
            .to_string()
            .contains("No migration needed"));
    }

    #[test]
    fn migrate_ok() {
        let test_info = TestInfo::init_with_version(Some("1.0.3"));

        let contract = Box::new(
            ContractWrapper::new(execute, instantiate, query)
                .with_sudo(sudo)
                .with_migrate_empty(migrate),
        );

        let new_code_id = test_info
            .app_mut()
            .store_code_with_creator(test_info.creator().addr(), contract);

        let migrate_response = test_info.app_mut().migrate_contract(
            test_info.creator().addr(),
            test_info.contract_addr(),
            &Empty {},
            new_code_id,
        );

        match migrate_response {
            Ok(response) => {
                let migrate_event = response.events.iter().find(|e| e.ty == "wasm-seda-contract");
                let migrate_event = migrate_event.unwrap();
                let migrate_event_attributes = migrate_event
                    .attributes
                    .iter()
                    .map(|a| (a.key.clone(), a.value.clone()))
                    .collect::<HashMap<String, String>>();

                assert_eq!(migrate_event_attributes.get("action"), Some(&"migrate".to_string()));
                assert_eq!(
                    migrate_event_attributes.get("current_version"),
                    Some(&"1.0.3".to_string())
                );
                assert_eq!(
                    migrate_event_attributes.get("target_version"),
                    Some(&CONTRACT_VERSION.to_string())
                );
            }
            Err(e) => panic!("Migrate failed: {}", e),
        }
    }
}
