use common::msg::PostDataRequestArgs;
use common::state::Reveal;
use common::types::Bytes;
use common::types::Hash;
use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
use data_requests::state::DataRequestInputs;
use data_requests::utils::hash_data_request;
use proxy_contract::msg::ProxyExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use sha3::Keccak256;
use cw_utils::parse_execute_response_data;

pub const USER: &str = "user";
pub const EXECUTOR_1: &str = "executor1";
pub const EXECUTOR_2: &str = "executor2";
pub const EXECUTOR_3: &str = "executor3";
const ADMIN: &str = "admin";
pub const NATIVE_DENOM: &str = "seda";

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<proxy_contract::msg::ProxyExecuteMsg>>(
        &self,
        msg: T,
    ) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn call_with_deposit<T: Into<proxy_contract::msg::ProxyExecuteMsg>>(
        &self,
        msg: T,
        amount: u128,
    ) -> StdResult<CosmosMsg> {
        let coin = Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: amount.into(),
        };
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![coin],
        }
        .into())
    }

    pub fn sudo<T: Into<proxy_contract::msg::ProxySudoMsg>>(
        &self,
        msg: T,
    ) -> cw_multi_test::SudoMsg {
        let msg = to_binary(&msg.into()).unwrap();
        cw_multi_test::SudoMsg::Wasm(cw_multi_test::WasmSudo {
            contract_addr: self.addr().into(),
            msg,
        })
    }

    pub fn sudo_staking<T: Into<staking::msg::StakingSudoMsg>>(
        &self,
        msg: T,
    ) -> cw_multi_test::SudoMsg {
        let msg = to_binary(&msg.into()).unwrap();
        cw_multi_test::SudoMsg::Wasm(cw_multi_test::WasmSudo {
            contract_addr: self.addr().into(),
            msg,
        })
    }
}

pub fn proxy_contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        proxy_contract::contract::execute,
        proxy_contract::contract::instantiate,
        proxy_contract::contract::query,
    )
    .with_sudo(proxy_contract::contract::sudo)
    .with_reply(proxy_contract::contract::reply);
    Box::new(contract)
}

pub fn data_requests_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        data_requests::contract::execute,
        data_requests::contract::instantiate,
        data_requests::contract::query,
    );
    Box::new(contract)
}

pub fn staking_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        staking::contract::execute,
        staking::contract::instantiate,
        staking::contract::query,
    )
    .with_sudo(staking::contract::sudo);
    Box::new(contract)
}

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }],
            )
            .unwrap();
    })
}

pub fn send_tokens(app: &mut App, from: &str, to: &str, amount: u128) {
    let coin = Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: amount.into(),
    };
    let cosmos_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: to.to_string(),
        amount: vec![coin],
    });
    app.execute(Addr::unchecked(from), cosmos_msg).unwrap();
}

pub fn proper_instantiate() -> (App, CwTemplateContract) {
    let mut app = mock_app();

    // instantiate proxy-contract
    let proxy_contract_template_id = app.store_code(proxy_contract_template());
    let msg = proxy_contract::msg::InstantiateMsg {
        token: NATIVE_DENOM.to_string(),
    };
    let proxy_contract_addr = app
        .instantiate_contract(
            proxy_contract_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();
    let proxy_template_contract = CwTemplateContract(proxy_contract_addr.clone());

    // instantiate staking
    let staking_template_id = app.store_code(staking_template());
    let msg = common::msg::InstantiateMsg {
        token: NATIVE_DENOM.to_string(),
        proxy: proxy_contract_addr.to_string(),
    };
    let staking_contract_addr = app
        .instantiate_contract(
            staking_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    // instantiate data-requests
    let data_requests_template_id = app.store_code(data_requests_template());
    let msg = common::msg::InstantiateMsg {
        token: NATIVE_DENOM.to_string(),
        proxy: proxy_contract_addr.to_string(),
    };
    let data_requests_contract_addr = app
        .instantiate_contract(
            data_requests_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    // set contract addresses on proxy-contract
    let msg = proxy_contract::msg::ProxyExecuteMsg::SetStaking {
        contract: staking_contract_addr.to_string(),
    };
    let cosmos_msg = proxy_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();
    let msg = proxy_contract::msg::ProxyExecuteMsg::SetDataRequests {
        contract: data_requests_contract_addr.to_string(),
    };
    let cosmos_msg = proxy_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

    (app, proxy_template_contract)
}

pub fn get_dr_id(res: AppResponse) -> String {
    let binary = parse_execute_response_data(&res.data.unwrap().0).unwrap().data.unwrap();

    let dr_id = String::from_utf8(binary.to_vec()).unwrap();

    // remove first and last char (they are quotes)
    let dr_id = &dr_id[1..dr_id.len() - 1];

    dr_id.to_string()
}

pub fn calculate_commitment(reveal: &str, salt: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(reveal.as_bytes());
    hasher.update(salt.as_bytes());
    let digest = hasher.finalize();
    format!("0x{}", hex::encode(digest))
}

pub fn helper_commit_result(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    dr_id: String,
    commitment: String,
    sender: Addr,
) -> Result<AppResponse, anyhow::Error> {
    let msg = ProxyExecuteMsg::CommitDataResult { dr_id, commitment };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn helper_reveal_result(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    dr_id: String,
    reveal: Reveal,
    sender: Addr,
) -> Result<AppResponse, anyhow::Error> {
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id: dr_id.to_string(),
        reveal,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn helper_post_dr(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    posted_dr: PostDataRequestArgs,
    sender: Addr,
) -> Result<AppResponse, anyhow::Error> {
    let msg = ProxyExecuteMsg::PostDataRequest { posted_dr };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn calculate_dr_id_and_args(
    nonce: u128,
    replication_factor: u16,
) -> (String, PostDataRequestArgs) {
    let dr_binary_id: Hash = "dr_binary_id".to_string();
    let tally_binary_id: Hash = "tally_binary_id".to_string();
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();

    // set by dr creator
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;

    // set by relayer and SEDA protocol
    let seda_payload: Bytes = Vec::new();
    let payback_address: Bytes = Vec::new();

    // memo
    let chain_id: u128 = 31337;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo = hasher.finalize().to_vec();

    let constructed_dr_input = DataRequestInputs {
        dr_binary_id: dr_binary_id.clone(),
        tally_binary_id: tally_binary_id.clone(),
        dr_inputs: dr_inputs.clone(),
        tally_inputs: tally_inputs.clone(),
        memo: memo.clone(),
        replication_factor,

        gas_price,
        gas_limit,

        seda_payload: seda_payload.clone(),
        payback_address: payback_address.clone(),
    };
    let constructed_dr_id = hash_data_request(constructed_dr_input);

    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        dr_id: constructed_dr_id.clone(),
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo,
        replication_factor,
        gas_price,
        gas_limit,
        seda_payload,
        payback_address,
    };

    (constructed_dr_id, posted_dr)
}
