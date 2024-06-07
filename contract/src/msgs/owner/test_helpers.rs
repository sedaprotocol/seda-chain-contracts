use super::{
    msgs::owner::{execute, query},
    *,
};
use crate::{TestExecutor, TestInfo};

impl TestInfo {
    #[track_caller]
    pub fn accept_ownership(&mut self, sender: &TestExecutor) -> Result<(), ContractError> {
        let msg = execute::accept_ownership::Execute {}.into();
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn get_owner(&self) -> Addr {
        self.query(query::QueryMsg::GetOwner {}).unwrap()
    }

    #[track_caller]
    pub fn get_pending_owner(&self) -> Option<Addr> {
        self.query(query::QueryMsg::GetPendingOwner {}).unwrap()
    }

    #[track_caller]
    pub fn transfer_ownership(&mut self, sender: &TestExecutor, new_owner: &TestExecutor) -> Result<(), ContractError> {
        let msg = execute::transfer_ownership::Execute {
            new_owner: new_owner.addr().into_string(),
        }
        .into();
        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn add_to_allowlist(&mut self, sender: &TestExecutor, public_key: PublicKey) -> Result<(), ContractError> {
        let msg = execute::add_to_allowlist::Execute { public_key }.into();

        self.execute(sender, &msg)
    }

    #[track_caller]
    pub fn remove_from_allowlist(&mut self, sender: &TestExecutor, public_key: PublicKey) -> Result<(), ContractError> {
        let msg = execute::remove_from_allowlist::Execute { public_key }.into();

        self.execute(sender, &msg)
    }
}
