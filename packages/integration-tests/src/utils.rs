use common::consts::INITIAL_MINIMUM_STAKE_TO_REGISTER;
use common::msg::PostDataRequestArgs;
use common::state::RevealBody;
use common::test_utils::TestExecutor;
use common::types::Bytes;
use common::types::Hash;
use common::types::Secpk256k1PublicKey;
use common::types::Signature;
use common::types::SimpleHash;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, StdResult, Uint128, WasmMsg,
};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
use cw_utils::parse_execute_response_data;
use data_requests::utils::string_to_hash;
use proxy_contract::msg::ProxyExecuteMsg;
use schemars::JsonSchema;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use sha3::Keccak256;

pub const USER: &str = "user";
pub const EXECUTOR_1: &str = "executor1";
const OWNER: &str = "owner";
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
        let msg = to_json_binary(&msg.into())?;
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
        let msg = to_json_binary(&msg.into())?;
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
        let msg = to_json_binary(&msg.into()).unwrap();
        cw_multi_test::SudoMsg::Wasm(cw_multi_test::WasmSudo {
            contract_addr: self.addr(),
            message: msg,
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
    );
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
            Addr::unchecked(OWNER),
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
        owner: OWNER.to_string(),
    };
    let staking_contract_addr = app
        .instantiate_contract(
            staking_template_id,
            Addr::unchecked(OWNER),
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
        owner: OWNER.to_string(),
    };
    let data_requests_contract_addr = app
        .instantiate_contract(
            data_requests_template_id,
            Addr::unchecked(OWNER),
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
    app.execute(Addr::unchecked(OWNER), cosmos_msg).unwrap();
    let msg = proxy_contract::msg::ProxyExecuteMsg::SetDataRequests {
        contract: data_requests_contract_addr.to_string(),
    };
    let cosmos_msg = proxy_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(OWNER), cosmos_msg).unwrap();

    (app, proxy_template_contract)
}

pub fn get_dr_id(res: AppResponse) -> Hash {
    let binary = parse_execute_response_data(&res.data.unwrap().0)
        .unwrap()
        .data
        .unwrap();

    binary.0.try_into().unwrap()
}

// So we can have the hash and the bytes to sign it.
pub fn reveal_hash(reveal: &RevealBody, salt: Option<&'static str>) -> (Hash, Vec<Vec<u8>>) {
    let mut reveal_hasher = Keccak256::new();
    reveal_hasher.update(&reveal.reveal);
    let reveal_hash = reveal_hasher.finalize();

    let salt = if let Some(salt_str) = salt {
        string_to_hash(salt_str)
    } else {
        reveal.salt
    };

    let mut reveal_body_hasher = Keccak256::new();
    reveal_body_hasher.update(salt);
    reveal_body_hasher.update(reveal.exit_code.to_be_bytes());
    reveal_body_hasher.update(reveal.gas_used.to_be_bytes());
    reveal_body_hasher.update(reveal_hash);

    let bytes = vec![
        reveal.salt.to_vec(),
        reveal.exit_code.to_be_bytes().to_vec(),
        reveal.gas_used.to_be_bytes().to_vec(),
        reveal_hash.to_vec(),
    ];
    (reveal_body_hasher.finalize().into(), bytes)
}

pub fn helper_reg_dr_executor(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    executor: &TestExecutor,
    memo: Option<String>,
) -> Result<AppResponse, anyhow::Error> {
    let sender = Addr::unchecked(executor.name);
    let contract_call_bytes = "register_data_request_executor".as_bytes().to_vec();
    let signature = if let Some(m) = memo.as_ref() {
        executor.sign([
            contract_call_bytes,
            sender.as_bytes().to_vec(),
            m.simple_hash().to_vec(),
        ])
    } else {
        executor.sign([contract_call_bytes, sender.as_bytes().to_vec()])
    };
    let msg = ProxyExecuteMsg::RegisterDataRequestExecutor { signature, memo };
    let cosmos_msg = proxy_contract
        .call_with_deposit(msg, INITIAL_MINIMUM_STAKE_TO_REGISTER)
        .unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn helper_commit_result(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    dr_id: Hash,
    commitment: Hash,
    public_key: Secpk256k1PublicKey,
    sender: Addr,
) -> Result<AppResponse, anyhow::Error> {
    let msg = ProxyExecuteMsg::CommitDataResult {
        dr_id,
        commitment,
        public_key,
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn helper_reveal_result(
    app: &mut App,
    proxy_contract: CwTemplateContract,
    dr_id: Hash,
    reveal: RevealBody,
    signature: Signature,
    sender: Addr,
) -> Result<AppResponse, anyhow::Error> {
    let msg = ProxyExecuteMsg::RevealDataResult {
        dr_id,
        reveal,
        signature,
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
    let msg = ProxyExecuteMsg::PostDataRequest {
        posted_dr,
        seda_payload: Vec::new(),
        payback_address: Vec::new(),
    };
    let cosmos_msg = proxy_contract.call(msg).unwrap();
    app.execute(sender, cosmos_msg.clone())
}

pub fn calculate_dr_id_and_args(nonce: u128, replication_factor: u16) -> PostDataRequestArgs {
    let dr_binary_id: Hash = string_to_hash("dr_binary_id");
    let tally_binary_id: Hash = string_to_hash("tally_binary_id");
    let dr_inputs: Bytes = Vec::new();
    let tally_inputs: Bytes = Vec::new();

    // set by dr creator
    let gas_price: u128 = 10;
    let gas_limit: u128 = 10;
    let _tally_gas_limit: u128 = 10;

    // memo
    let chain_id: u128 = 31337;
    let mut hasher = Keccak256::new();
    hasher.update(chain_id.to_be_bytes());
    hasher.update(nonce.to_be_bytes());
    let memo = hasher.finalize().to_vec();
    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };

    let posted_dr: PostDataRequestArgs = PostDataRequestArgs {
        version,
        dr_binary_id,
        tally_binary_id,
        dr_inputs,
        tally_inputs,
        memo,
        replication_factor,
        gas_price,
        gas_limit,
    };

    posted_dr
}
