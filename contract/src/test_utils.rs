use std::collections::HashMap;

use cosmwasm_std::{
    coins,
    from_json,
    testing::{mock_info, MockApi},
    to_json_binary,
    Addr,
    MessageInfo,
    WasmMsg,
};
use cw_multi_test::{no_init, App, AppBuilder, ContractWrapper, Executor};
use cw_utils::parse_instantiate_response_data;
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use serde::{de::DeserializeOwned, Serialize};
use sha3::{Digest, Keccak256};
use vrf_rs::Secp256k1Sha256;

use crate::{
    contract::*,
    error::ContractError,
    msgs::{ExecuteMsg, InstantiateMsg},
    types::{Hash, PublicKey},
};

pub struct TestInfo {
    app:           App,
    contract_addr: Addr,
    executors:     HashMap<&'static str, TestExecutor>,
}

impl TestInfo {
    pub fn init() -> Self {
        let mut app = AppBuilder::default()
            .with_api(MockApi::default().with_prefix("seda"))
            .build(no_init);
        let contract = Box::new(ContractWrapper::new(execute, instantiate, query).with_sudo(sudo));

        let creator_addr = app.api().addr_make("creator");
        let creator = TestExecutor::new("creator", creator_addr.clone(), Some(1_000_000));

        let code_id = app.store_code_with_creator(creator.addr(), contract);
        let init_msg = to_json_binary(&InstantiateMsg {
            token: "aseda".to_string(),
            owner: creator.addr().into_string(),
        })
        .unwrap();

        let mut executors = HashMap::new();
        executors.insert("creator", creator.clone());

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id,
            msg: init_msg,
            funds: vec![],
            label: "label".into(),
        };
        let res = app.execute(creator.addr, msg.into()).unwrap();
        let parsed = parse_instantiate_response_data(res.data.unwrap().as_slice()).unwrap();
        assert!(parsed.data.is_none());

        Self {
            app,
            contract_addr: Addr::unchecked(parsed.contract_address),
            executors,
        }
    }

    pub fn new_executor(&mut self, name: &'static str, amount: Option<u128>) -> TestExecutor {
        let addr = self.app.api().addr_make(name);
        let executor = TestExecutor::new(name, addr, amount);
        self.executors.insert(name, executor);
        self.executor(name).clone()
    }

    #[track_caller]
    pub fn executor(&self, name: &'static str) -> &TestExecutor {
        self.executors.get(name).unwrap()
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn creator(&self) -> TestExecutor {
        self.executor("creator").clone()
    }

    pub fn contract_addr(&self) -> &str {
        self.contract_addr.as_str()
    }

    pub fn query<M: Serialize, R: DeserializeOwned>(&self, msg: M) -> Result<R, cosmwasm_std::StdError> {
        self.app.wrap().query_wasm_smart(self.contract_addr(), &msg)
    }

    #[track_caller]
    pub fn execute<R: DeserializeOwned>(
        &mut self,
        sender: &TestExecutor,
        msg: &ExecuteMsg,
    ) -> Result<R, ContractError> {
        let res = self
            .app
            .execute_contract(sender.addr(), self.contract_addr.clone(), msg, &[])
            .map_err(|e| e.downcast_ref::<ContractError>().cloned().unwrap())?;

        Ok(match res.data {
            Some(data) => from_json(data).unwrap(),
            None => from_json(to_json_binary(&serde_json::Value::Null).unwrap()).unwrap(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TestExecutor {
    pub name:    &'static str,
    addr:        Addr,
    signing_key: SigningKey,
    public_key:  PublicKey,
    info:        MessageInfo,
}

impl TestExecutor {
    pub fn new(name: &'static str, addr: Addr, amount: Option<u128>) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = verifying_key.to_encoded_point(true).to_bytes();
        let coins = if let Some(amount) = amount {
            coins(amount, "aseda")
        } else {
            vec![]
        };
        TestExecutor {
            name,
            addr,
            signing_key,
            public_key: public_key.to_vec(),
            info: mock_info(name, &coins),
        }
    }

    pub fn addr(&self) -> Addr {
        self.addr.clone()
    }

    pub fn pub_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn info(&self) -> MessageInfo {
        self.info.clone()
    }

    pub fn set_amount(&mut self, amount: u128) {
        self.info = mock_info(self.name, &coins(amount, "aseda"));
    }

    pub fn _remove_coins(&mut self) {
        self.info = mock_info(self.name, &[]);
    }

    pub fn prove(&self, hash: &[u8]) -> Vec<u8> {
        let vrf = Secp256k1Sha256::default();
        vrf.prove(&self.signing_key.to_bytes(), hash).unwrap()
    }

    pub fn salt(&self) -> Hash {
        let mut hasher = Keccak256::new();
        hasher.update(self.name);
        hasher.finalize().into()
    }
}

#[test]
fn instantiate_works() {
    TestInfo::init();
}
