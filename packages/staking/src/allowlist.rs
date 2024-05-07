pub mod allow_list {
    use crate::state::{ALLOWLIST, OWNER};
    use crate::utils::validate_sender;
    use common::error::ContractError;
    #[cfg(not(feature = "library"))]
    use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response};

    pub fn add_to_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        sender: Option<String>,
        address: Addr,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if sender != owner {
            return Err(ContractError::NotOwner);
        }

        // add the address to the allowlist
        ALLOWLIST.save(deps.storage, address, &true)?;

        Ok(Response::new())
    }

    pub fn remove_from_allowlist(
        deps: DepsMut,
        info: MessageInfo,
        sender: Option<String>,
        address: Addr,
    ) -> Result<Response, ContractError> {
        let sender = validate_sender(&deps, info.sender, sender)?;

        // require the sender to be the OWNER
        let owner = OWNER.load(deps.storage)?;
        if sender != owner {
            return Err(ContractError::NotOwner);
        }

        // remove the address from the allowlist
        ALLOWLIST.remove(deps.storage, address);

        Ok(Response::new())
    }
}

#[cfg(test)]
mod executers_tests {
    use crate::helpers::helper_add_to_allowlist;
    use crate::helpers::helper_register_executor;
    use crate::helpers::helper_remove_from_allowlist;
    use crate::helpers::helper_set_staking_config;
    use crate::helpers::helper_unregister_executor;
    use crate::helpers::helper_unstake;
    use crate::helpers::helper_withdraw;
    use crate::helpers::instantiate_staking_contract;
    use common::error::ContractError;
    use common::state::StakingConfig;
    use common::types::Signature;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_info};

    #[test]
    pub fn allowlist_works() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate_staking_contract(deps.as_mut(), info).unwrap();

        // update the config with allowlist enabled
        let info = mock_info("owner", &coins(0, "token"));
        let new_config = StakingConfig {
            minimum_stake_to_register: 100,
            minimum_stake_for_committee_eligibility: 200,
            allowlist_enabled: true,
        };
        let res = helper_set_staking_config(deps.as_mut(), info, new_config);
        assert!(res.is_ok());

        // alice tries to register a data request executor, but she's not on the allowlist
        let info = mock_info("alice", &coins(100, "token"));
        let res = helper_register_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            Signature::new([0; 65]),
            None,
            None,
        );
        assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

        // add alice to the allowlist
        let info = mock_info("owner", &coins(0, "token"));
        let res = helper_add_to_allowlist(deps.as_mut(), info, "alice".to_string(), None);
        assert!(res.is_ok());

        // now alice can register a data request executor
        let info = mock_info("alice", &coins(100, "token"));
        let res = helper_register_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            Signature::new([0; 65]),
            None,
            None,
        );
        assert!(res.is_ok());

        // alice unstakes, withdraws, then unregisters herself
        let info = mock_info("alice", &coins(0, "token"));
        let _res = helper_unstake(
            deps.as_mut(),
            info.clone(),
            vec![0; 33],
            Signature::new([0; 65]),
            100,
            None,
        );
        let info = mock_info("alice", &coins(0, "token"));
        let _res = helper_withdraw(
            deps.as_mut(),
            info.clone(),
            vec![0; 33],
            Signature::new([0; 65]),
            100,
            None,
        );
        let info = mock_info("alice", &coins(0, "token"));
        let res = helper_unregister_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            Signature::new([0; 65]),
            None,
        );
        println!("{:?}", res);
        assert!(res.is_ok());

        // remove alice from the allowlist
        let info = mock_info("owner", &coins(0, "token"));
        let res = helper_remove_from_allowlist(deps.as_mut(), info, "alice".to_string(), None);
        assert!(res.is_ok());

        // now alice can't register a data request executor
        let info = mock_info("alice", &coins(2, "token"));
        let res = helper_register_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            Signature::new([0; 65]),
            None,
            None,
        );
        assert!(res.is_err_and(|x| x == ContractError::NotOnAllowlist));

        // update the config to disable the allowlist
        let info = mock_info("owner", &coins(0, "token"));
        let new_config = StakingConfig {
            minimum_stake_to_register: 100,
            minimum_stake_for_committee_eligibility: 200,
            allowlist_enabled: false,
        };
        let res = helper_set_staking_config(deps.as_mut(), info, new_config);
        assert!(res.is_ok());

        // now alice can register a data request executor
        let info = mock_info("alice", &coins(100, "token"));
        let res = helper_register_executor(
            deps.as_mut(),
            info,
            vec![0; 33],
            Signature::new([0; 65]),
            None,
            None,
        );
        assert!(res.is_ok());
    }
}
