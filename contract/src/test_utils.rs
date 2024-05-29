use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    DepsMut,
    MessageInfo,
    Response,
};
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use sha3::{Digest, Keccak256};
use vrf_rs::Secp256k1Sha256;

use crate::{
    contract::instantiate,
    error::ContractError,
    msgs::InstantiateMsg,
    types::{Hash, PublicKey},
};

pub struct TestExecutor {
    pub name:    &'static str,
    signing_key: SigningKey,
    public_key:  PublicKey,
    info:        MessageInfo,
}

impl TestExecutor {
    pub fn new(name: &'static str, amount: Option<u128>) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = verifying_key.to_encoded_point(true).to_bytes();
        let coins = if let Some(amount) = amount {
            coins(amount, "token")
        } else {
            vec![]
        };
        TestExecutor {
            name,
            signing_key,
            public_key: public_key.to_vec(),
            info: mock_info(name, &coins),
        }
    }

    pub fn pub_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn info(&self) -> MessageInfo {
        self.info.clone()
    }

    pub fn set_amount(&mut self, amount: u128) {
        self.info = mock_info(self.name, &coins(amount, "token"));
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

pub fn instantiate_contract(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        token: "token".to_string(),
        owner: "owner".to_string(),
    };
    instantiate(deps, mock_env(), info, msg)
}
