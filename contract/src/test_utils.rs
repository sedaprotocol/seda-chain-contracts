use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use cosmwasm_std::{coins, from_json, testing::MockApi, to_json_binary, Addr, Empty, StdError};
use cw_multi_test::{AppBuilder, ContractWrapper, Executor, StargateAccepting};
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use num_bigfloat::BigFloat;
use seda_common::{
    msgs::{staking::StakingConfig, *},
    types::ToHexStr,
};
use serde::{de::DeserializeOwned, Serialize};
use vrf_rs::Secp256k1Sha256;

use crate::{contract::*, error::ContractError, types::PublicKey};

pub fn new_public_key() -> (SigningKey, PublicKey) {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    let public_key = verifying_key.to_encoded_point(true).as_bytes().try_into().unwrap();

    (signing_key, PublicKey(public_key))
}

pub type App = cw_multi_test::App<
    cw_multi_test::BankKeeper,
    MockApi,
    cosmwasm_std::testing::MockStorage,
    cw_multi_test::FailingModule<Empty, Empty, Empty>,
    cw_multi_test::WasmKeeper<Empty, Empty>,
    cw_multi_test::StakeKeeper,
    cw_multi_test::DistributionKeeper,
    cw_multi_test::IbcFailingModule,
    cw_multi_test::GovFailingModule,
    StargateAccepting,
>;

pub struct TestInfo {
    app:           Rc<RefCell<App>>,
    contract_addr: Addr,
    accounts:      Rc<RefCell<HashMap<&'static str, TestAccount>>>,
    chain_id:      String,
}

pub fn seda_to_aseda(amount: BigFloat) -> u128 {
    let denominator: BigFloat = 10_f64.powf(18.0).into();

    (amount * denominator).to_u128().unwrap_or(u128::MAX)
}

pub fn aseda_to_seda(amount: u128) -> BigFloat {
    let denominator: BigFloat = 10_f64.powf(18.0).into();

    BigFloat::from_u128(amount) / denominator
}

impl TestInfo {
    pub fn init() -> Rc<Self> {
        let mut creator_addr = Addr::unchecked("creator");
        let app = Rc::new(RefCell::new(
            AppBuilder::default()
                .with_stargate(StargateAccepting)
                .with_api(MockApi::default().with_prefix("seda"))
                .build(|router, api, storage| {
                    creator_addr = api.addr_make("creator");
                    router
                        .bank
                        .init_balance(storage, &creator_addr, coins(1_000_000, "aseda"))
                        .unwrap();
                }),
        ));
        let contract = Box::new(ContractWrapper::new(execute, instantiate, query).with_sudo(sudo));
        let chain_id = "seda_test".to_string();

        let code_id = app.borrow_mut().store_code_with_creator(creator_addr.clone(), contract);
        let init_msg = &InstantiateMsg {
            token:          "aseda".to_string(),
            owner:          creator_addr.to_string(),
            chain_id:       chain_id.clone(),
            staking_config: Some(StakingConfig {
                minimum_stake:     1u128.into(),
                allowlist_enabled: false,
            }),
            timeout_config: None,
        };

        let contract_addr = app
            .borrow_mut()
            .instantiate_contract(code_id, creator_addr.clone(), &init_msg, &[], "core", None)
            .unwrap();

        let info = Rc::new(Self {
            app,
            contract_addr,
            accounts: Default::default(),
            chain_id,
        });

        let creator = TestAccount::new("creator", creator_addr, Rc::clone(&info));
        info.accounts.borrow_mut().insert("creator", creator);

        info.set_block_height(0);

        info
    }

    pub fn new_address(&self, name: &'static str) -> Addr {
        self.app.borrow().api().addr_make(name)
    }

    #[track_caller]
    pub fn new_account<S: Into<BigFloat>>(self: &Rc<Self>, name: &'static str, seda: S) -> TestAccount {
        let addr = self.new_address(name);
        let executor = TestAccount::new(name, addr, Rc::clone(self));
        self.accounts.borrow_mut().insert(name, executor);
        let executor = self.executor(name);

        let balance = seda_to_aseda(seda.into());

        self.app.borrow_mut().init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &executor.addr, coins(balance, "aseda"))
                .unwrap();
        });

        executor
    }

    #[track_caller]
    pub fn new_executor<S: Into<BigFloat>>(self: &Rc<Self>, name: &'static str, seda: S, stake: u128) -> TestAccount {
        let executor = self.new_account(name, seda);

        // stake if provided
        executor.stake(stake).unwrap();
        executor
    }

    #[track_caller]
    pub fn new_executor_with_memo(
        self: &Rc<Self>,
        name: &'static str,
        balance: u128,
        stake: u128,
        memo: &'static str,
    ) -> TestAccount {
        let addr = self.new_address(name);
        let executor = TestAccount::new(name, addr, Rc::clone(self));
        self.accounts.borrow_mut().insert(name, executor);
        let executor = self.accounts.borrow().get(name).unwrap().clone();

        self.app.borrow_mut().init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &executor.addr, coins(balance, "aseda"))
                .unwrap();
        });

        // stake if provided
        executor.stake_with_memo(stake, memo).unwrap();
        executor
    }

    #[track_caller]
    pub fn executor(self: &Rc<Self>, name: &'static str) -> TestAccount {
        let accounts = self.accounts.borrow();
        accounts.get(name).unwrap().clone()
    }

    #[track_caller]
    pub fn executor_balance(&self, name: &'static str) -> u128 {
        let executor = self.accounts.borrow();
        let executor = executor.get(name).unwrap();
        self.app()
            .wrap()
            .query_balance(executor.addr(), "aseda")
            .unwrap()
            .amount
            .u128()
    }

    pub fn app(&self) -> Ref<'_, App> {
        self.app.borrow()
    }

    pub fn app_mut(&mut self) -> RefMut<'_, App> {
        self.app.borrow_mut()
    }

    pub fn chain_id(&self) -> &str {
        self.chain_id.as_str()
    }

    pub fn block_height(&self) -> u64 {
        self.app.borrow().block_info().height
    }

    pub fn set_block_height(&self, height: u64) {
        self.app.borrow_mut().update_block(|b| {
            b.height = height;
        });
    }

    pub fn creator(self: &Rc<Self>) -> TestAccount {
        self.executor("creator")
    }

    pub fn contract_addr(&self) -> Addr {
        self.contract_addr.clone()
    }

    pub fn contract_addr_str(&self) -> &str {
        self.contract_addr.as_str()
    }

    pub fn contract_addr_bytes(&self) -> &[u8] {
        self.contract_addr.as_bytes()
    }

    pub fn query<M: Serialize, R: DeserializeOwned>(&self, msg: M) -> Result<R, cosmwasm_std::StdError> {
        self.app
            .borrow()
            .wrap()
            .query_wasm_smart(self.contract_addr_str(), &msg)
    }

    #[track_caller]
    pub fn sudo<R: DeserializeOwned>(&self, msg: &SudoMsg) -> Result<R, ContractError> {
        let res = self
            .app
            .borrow_mut()
            .wasm_sudo(self.contract_addr.clone(), msg)
            .map_err(|e| {
                if e.downcast_ref::<ContractError>().is_some() {
                    e.downcast().unwrap()
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
    pub fn execute<R: DeserializeOwned>(&self, sender: &TestAccount, msg: &ExecuteMsg) -> Result<R, ContractError> {
        let res = self
            .app
            .borrow_mut()
            .execute_contract(sender.addr(), self.contract_addr.clone(), msg, &[])
            .map_err(|e| {
                if e.downcast_ref::<ContractError>().is_some() {
                    e.downcast().unwrap()
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
        &self,
        sender: &TestAccount,
        msg: &ExecuteMsg,
        amount: u128,
    ) -> Result<R, ContractError> {
        let res = self
            .app
            .borrow_mut()
            .execute_contract(sender.addr(), self.contract_addr.clone(), msg, &coins(amount, "aseda"))
            .map_err(|e| {
                if e.downcast_ref::<ContractError>().is_some() {
                    e.downcast().unwrap()
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
}

#[derive(Clone)]
pub struct TestAccount {
    pub name:      &'static str,
    addr:          Addr,
    signing_key:   SigningKey,
    public_key:    PublicKey,
    pub test_info: Rc<TestInfo>,
}

impl TestAccount {
    fn new(name: &'static str, addr: Addr, test_info: Rc<TestInfo>) -> Self {
        let (signing_key, public_key) = new_public_key();

        TestAccount {
            name,
            addr,
            signing_key,
            public_key,
            test_info,
        }
    }

    pub fn addr(&self) -> Addr {
        self.addr.clone()
    }

    pub fn pub_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn pub_key_hex(&self) -> String {
        self.public_key.to_hex()
    }

    pub fn sign_key(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    pub fn prove(&self, hash: &[u8]) -> Vec<u8> {
        let vrf = Secp256k1Sha256::default();
        vrf.prove(&self.signing_key.to_bytes(), hash).unwrap()
    }

    pub fn prove_hex(&self, hash: &[u8]) -> String {
        self.prove(hash).to_hex()
    }
}

#[test]
fn instantiate_works() {
    TestInfo::init();
}
