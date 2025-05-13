use super::{
    msgs::owner::{execute, query},
    *,
};
use crate::TestAccount;

impl TestAccount {
    #[track_caller]
    pub fn accept_ownership(&self) -> Result<(), ContractError> {
        let msg = execute::accept_ownership::Execute {}.into();
        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn get_owner(&self) -> Addr {
        self.test_info.query(query::QueryMsg::GetOwner {}).unwrap()
    }

    #[track_caller]
    pub fn get_pending_owner(&self) -> Option<Addr> {
        self.test_info.query(query::QueryMsg::GetPendingOwner {}).unwrap()
    }

    #[track_caller]
    pub fn transfer_ownership(&self, new_owner: &TestAccount) -> Result<(), ContractError> {
        let msg = execute::transfer_ownership::Execute {
            new_owner: new_owner.addr().into_string(),
        }
        .into();
        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn add_to_allowlist(&self, public_key: PublicKey) -> Result<(), ContractError> {
        let msg = execute::add_to_allowlist::Execute {
            public_key: public_key.to_hex(),
        }
        .into();

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn get_allowlist(&self) -> Vec<String> {
        self.test_info.query(query::QueryMsg::GetAllowList {}).unwrap()
    }

    #[track_caller]
    pub fn remove_from_allowlist(&self, public_key: PublicKey) -> Result<(), ContractError> {
        let msg = execute::remove_from_allowlist::Execute {
            public_key: public_key.to_hex(),
        }
        .into();

        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn pause(&self) -> Result<(), ContractError> {
        let msg = execute::pause::Execute {}.into();
        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn unpause(&self) -> Result<(), ContractError> {
        let msg = execute::unpause::Execute {}.into();
        self.test_info.execute(self, &msg)
    }

    #[track_caller]
    pub fn is_paused(&self) -> bool {
        self.test_info.query(query::QueryMsg::IsPaused {}).unwrap()
    }
}
