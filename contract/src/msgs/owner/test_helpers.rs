use super::*;
use crate::{TestExecutor, TestInfo};

// pub fn add_to_allowlist(deps: DepsMut, info: MessageInfo, public_key: PublicKey) -> Result<Response, ContractError> {
//     let msg = add_to_allowlist::Execute { public_key };

//     execute(deps, mock_env(), info, msg.into())
// }

// pub fn remove_from_allowlist(
//     deps: DepsMut,
//     info: MessageInfo,
//     public_key: PublicKey,
// ) -> Result<Response, ContractError> {
//     let msg = remove_from_allowlist::Execute { public_key };

//     execute(deps, mock_env(), info, msg.into())
// }

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
        dbg!(to_json_string(&msg).unwrap());
        self.execute(sender, &msg)
    }
}
