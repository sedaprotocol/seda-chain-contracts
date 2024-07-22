use std::collections::HashMap;

use cosmwasm_std::{
    coins,
    from_json,
    testing::{mock_info, MockApi},
    to_json_binary,
    Addr,
    MessageInfo,
    StdError,
    Uint128,
    WasmMsg,
};
use cw_multi_test::{no_init, App, AppBuilder, BankSudo, ContractWrapper, Executor};
use cw_utils::parse_instantiate_response_data;
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use seda_common::{msgs::*, types::ToHexStr};
use serde::{de::DeserializeOwned, Serialize};
use sha3::{Digest, Keccak256};
use vrf_rs::Secp256k1Sha256;

use crate::{common_types::Hash, contract::*, error::ContractError, types::PublicKey};

pub struct TestInfo {
    app:           App,
    contract_addr: Addr,
    executors:     HashMap<&'static str, TestExecutor>,
    chain_id:      String,
}

impl TestInfo {
    pub fn init() -> Self {
        let mut app = AppBuilder::default()
            .with_api(MockApi::default().with_prefix("seda"))
            .build(no_init);
        let contract = Box::new(ContractWrapper::new(execute, instantiate, query).with_sudo(sudo));

        let creator_addr = app.api().addr_make("creator");
        let creator = TestExecutor::new("creator", creator_addr.clone(), Some(1_000_000));

        let chain_id = "seda_test".to_string();

        let code_id = app.store_code_with_creator(creator.addr(), contract);
        let init_msg = to_json_binary(&InstantiateMsg {
            token:    "aseda".to_string(),
            owner:    creator.addr().into_string(),
            chain_id: chain_id.clone(),
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

        let mut info = Self {
            app,
            contract_addr: Addr::unchecked(parsed.contract_address),
            executors,
            chain_id,
        };

        info.set_block_height(0);

        info
    }

    pub fn new_executor(&mut self, name: &'static str, amount: Option<u128>) -> TestExecutor {
        let addr = self.app.api().addr_make(name);
        let executor = TestExecutor::new(name, addr, amount);
        if let Some(amount) = amount {
            self.app
                .sudo(
                    BankSudo::Mint {
                        to_address: executor.addr().into_string(),
                        amount:     coins(amount, "aseda"),
                    }
                    .into(),
                )
                .unwrap();
        }

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

    pub fn chain_id(&self) -> &[u8] {
        self.chain_id.as_bytes()
    }

    pub fn block_height(&mut self) -> u64 {
        self.app.block_info().height
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.app.update_block(|b| {
            b.height = height;
        });
    }

    pub fn creator(&self) -> TestExecutor {
        self.executor("creator").clone()
    }

    pub fn contract_addr(&self) -> &str {
        self.contract_addr.as_str()
    }

    pub fn contract_addr_bytes(&self) -> &[u8] {
        self.contract_addr.as_bytes()
    }

    pub fn query<M: Serialize, R: DeserializeOwned>(&self, msg: M) -> Result<R, cosmwasm_std::StdError> {
        self.app.wrap().query_wasm_smart(self.contract_addr(), &msg)
    }

    #[track_caller]
    pub fn sudo<R: DeserializeOwned>(&mut self, msg: &SudoMsg) -> Result<R, ContractError> {
        let res = self.app.wasm_sudo(self.contract_addr.clone(), msg).map_err(|e| {
            if let Some(c_err) = e.downcast_ref::<ContractError>() {
                c_err.clone()
            } else if let Some(s_err) = e.downcast_ref::<StdError>() {
                return ContractError::Std(s_err.to_string());
            } else {
                ContractError::Dbg(e.to_string())
            }
        });

        Ok(match res?.data {
            Some(data) => from_json(data).unwrap(),
            None => from_json(to_json_binary(&serde_json::Value::Null).unwrap()).unwrap(),
        })
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
            .map_err(|e| {
                if let Some(c_err) = e.downcast_ref::<ContractError>() {
                    c_err.clone()
                } else if let Some(s_err) = e.downcast_ref::<StdError>() {
                    return ContractError::Std(s_err.to_string());
                } else {
                    ContractError::Dbg(e.to_string())
                }
            });

        Ok(match res?.data {
            Some(data) => from_json(data).unwrap(),
            None => from_json(to_json_binary(&serde_json::Value::Null).unwrap()).unwrap(),
        })
    }

    #[track_caller]
    pub fn execute_with_funds<R: DeserializeOwned>(
        &mut self,
        sender: &mut TestExecutor,
        msg: &ExecuteMsg,
        amount: u128,
    ) -> Result<R, ContractError> {
        let res = self
            .app
            .execute_contract(sender.addr(), self.contract_addr.clone(), msg, &coins(amount, "aseda"))
            .map_err(|e| {
                if let Some(c_err) = e.downcast_ref::<ContractError>() {
                    c_err.clone()
                } else if let Some(s_err) = e.downcast_ref::<StdError>() {
                    return ContractError::Std(s_err.to_string());
                } else {
                    ContractError::Dbg(e.to_string())
                }
            });

        Ok(match res?.data {
            Some(data) => {
                sender.sub_seda(amount);
                from_json(data).unwrap()
            }
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
    fn new(name: &'static str, addr: Addr, amount: Option<u128>) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = verifying_key.to_encoded_point(true).as_bytes().try_into().unwrap();
        let coins = if let Some(amount) = amount {
            coins(amount, "aseda")
        } else {
            vec![]
        };
        TestExecutor {
            name,
            addr,
            signing_key,
            public_key,
            info: mock_info(name, &coins),
        }
    }

    pub fn addr(&self) -> Addr {
        self.addr.clone()
    }

    pub fn pub_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn pub_key_hex(&self) -> String {
        self.public_key.to_hex()
    }

    pub fn info(&self) -> MessageInfo {
        self.info.clone()
    }

    fn funds(&self) -> Uint128 {
        self.info.funds.iter().find(|c| c.denom == "aseda").unwrap().amount
    }

    pub fn add_seda(&mut self, amount: u128) {
        self.info = mock_info(self.name, &coins(self.funds().u128() + amount, "aseda"));
    }

    pub fn sub_seda(&mut self, amount: u128) {
        self.info = mock_info(self.name, &coins(self.funds().u128() - amount, "aseda"));
    }

    pub fn prove(&self, hash: &[u8]) -> Vec<u8> {
        let vrf = Secp256k1Sha256::default();
        vrf.prove(&self.signing_key.to_bytes(), hash).unwrap()
    }

    pub fn prove_hex(&self, hash: &[u8]) -> String {
        self.prove(hash).to_hex()
    }

    pub fn salt(&self) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(self.name);
        let hash: Hash = hasher.finalize().into();
        hash.to_hex()
    }

    pub fn stake(&mut self, test_info: &mut TestInfo, amount: u128) -> Result<(), ContractError> {
        test_info.stake(self, None, amount)?;
        Ok(())
    }
}

#[test]
fn instantiate_works() {
    TestInfo::init();
}
