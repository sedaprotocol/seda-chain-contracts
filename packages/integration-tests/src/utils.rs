use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const USER: &str = "user";
pub const EXECUTOR_1: &str = "executor1";
pub const EXECUTOR_2: &str = "executor2";
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
}

pub fn proxy_contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        proxy_contract::contract::execute,
        proxy_contract::contract::instantiate,
        proxy_contract::contract::query,
    );
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
    let msg = staking::msg::InstantiateMsg {
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
    let msg = data_requests::msg::InstantiateMsg {
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
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    let msg = proxy_contract::msg::ProxyExecuteMsg::SetDataRequests {
        contract: data_requests_contract_addr.to_string(),
    };
    let cosmos_msg = proxy_template_contract.call(msg).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

    (app, proxy_template_contract)
}
