use std::cmp::Ordering;
use ethnum::{u256, AsU256};
use std::ops::{Add, Sub};
use std::string::ToString;
use crate::errors::{Result, GeneralError::ParseError};
use crate::errors::GeneralError::MathError;

pub const ADDRESS_LENGTH: usize = 20;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Address {
    addr: [u8; ADDRESS_LENGTH],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Address {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), ADDRESS_LENGTH, "invalid length");
        let mut ret = Address {
            addr: [0u8; ADDRESS_LENGTH]
        };
        ret.addr.copy_from_slice(bytes);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.addr)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.addr.into()
    }

    pub fn to_string(&self) -> String {
        self.to_hex()
    }

    pub fn eq(&self, other: &Address) -> bool {
        self.addr.eq(&other.addr)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum BalanceType {
    Native,
    HOPR
}

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Balance {
    value: u256,
    balance_type: BalanceType
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Balance {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn from_str(value: &str, balance_type: BalanceType) -> Self {
        Balance {
            value: u256::from_str_radix(value, 10).unwrap(),
            balance_type,
        }
    }

    pub fn balance_type(&self) -> BalanceType {
        self.balance_type
    }

    pub fn to_hex(&self) -> String { hex::encode(self.value().to_be_bytes()) }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn eq(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().eq(other.value())
    }

    pub fn serialize_value(&self) -> Box<[u8]> {
        Box::new(self.value().to_be_bytes())
    }

    pub fn lt(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().lt(other.value())
    }

    pub fn lte(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().lt(other.value()) || self.value().eq(other.value())
    }

    pub fn gt(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().gt(other.value())
    }

    pub fn gte(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().gt(other.value()) || self.value().eq(other.value())
    }

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Balance {
            value: self.value().add(other.value()),
            balance_type: self.balance_type
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Balance {
            value: self.value().add(amount.as_u256()),
            balance_type: self.balance_type
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Balance {
            value: self.value().sub(other.value()),
            balance_type: self.balance_type
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Balance {
            value: self.value().sub(amount.as_u256()),
            balance_type: self.balance_type,
        }
    }
}

impl Balance {
    pub fn new(value: u256, balance_type: BalanceType) -> Self {
        Balance {
            value, balance_type
        }
    }

    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn deserialize(data: &[u8], balance_type: BalanceType) -> Result<Balance> {
        Ok(Balance {
            value: u256::from_be_bytes(
                data.try_into().map_err(|_| ParseError)?),
            balance_type
        })
    }
}

pub const ETHEREUM_CHALLENGE_LENGTH: usize = 20;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct EthereumChallenge {
    challenge: [u8; ETHEREUM_CHALLENGE_LENGTH]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl EthereumChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), ETHEREUM_CHALLENGE_LENGTH);

        let mut ret = EthereumChallenge {
            challenge: [0u8; ETHEREUM_CHALLENGE_LENGTH]
        };
        ret.challenge.copy_from_slice(data);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.challenge)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.challenge.into()
    }

    pub fn eq(&self, other: &EthereumChallenge) -> bool {
        self.challenge.eq(&other.challenge)
    }
}

impl EthereumChallenge {
    pub fn deserialize(data: &[u8]) -> Result<EthereumChallenge> {
        if data.len() == ETHEREUM_CHALLENGE_LENGTH {
            Ok(EthereumChallenge::new(data))
        } else {
            Err(ParseError)
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct U256 {
    value: u256
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl U256 {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(value: &str) -> Self {
        U256 {
            value: u256::from_str_radix(value, 10)
                .expect("invalid decimal number string")
        }
    }

    pub fn to_hex(&self) -> String { hex::encode(self.value().to_be_bytes()) }

    pub fn serialize(&self) -> Box<[u8]> {
        self.value.to_be_bytes().into()
    }

    pub fn eq(&self, other: &U256) -> bool {
        self.value.eq(&other.value)
    }

    pub fn cmp(&self, other: &U256) -> i32 {
        match self.value.cmp(&other.value) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}

impl U256 {
    pub fn deserialize(data: &[u8]) -> Result<U256> {
        Ok(U256{
            value: u256::from_be_bytes(data.try_into().map_err(|_|ParseError)?)
        })
    }

    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn from_inverse_probability(inverse_prob: &u256) -> Result<U256> {
        let higest_prob = u256::MAX;
        if inverse_prob.gt(&u256::ZERO) {
            Ok(U256{
                value: higest_prob / inverse_prob
            })
        }
        else if inverse_prob.eq(&u256::ZERO) {
            Ok(U256 {
                value: higest_prob
            })
        }
        else {
            Err(MathError)
        }
    }
}

impl From<u256> for U256 {
    fn from(value: u256) -> Self {
        U256 { value }
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_tests() {
        let b = Balance::from_str("10", BalanceType::HOPR);
        assert_eq!("10".to_string(), b.to_string());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::primitives::{Balance, BalanceType, EthereumChallenge, U256};

    #[wasm_bindgen]
    impl Balance {
        pub fn deserialize_balance(data: &[u8], balance_type: BalanceType) -> JsResult<Balance> {
            ok_or_jserr!(Balance::deserialize(data, balance_type))
        }
    }

    #[wasm_bindgen]
    impl U256 {
        pub fn deserialize_u256(data: &[u8]) -> JsResult<U256> {
            ok_or_jserr!(U256::deserialize(data))
        }

        pub fn u256_from_inverse_probability(inverse_prob: &U256) -> JsResult<U256> {
            ok_or_jserr!(U256::from_inverse_probability(inverse_prob.value()))
        }
    }

    #[wasm_bindgen]
    impl EthereumChallenge {
        pub fn deserialize_challenge(data: &[u8]) -> JsResult<EthereumChallenge> {
            ok_or_jserr!(EthereumChallenge::deserialize(data))
        }
    }
}
