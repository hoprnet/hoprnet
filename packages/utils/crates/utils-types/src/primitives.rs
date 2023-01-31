use std::num::ParseIntError;
use ethnum::{u256, AsU256};
use std::ops::{Add, Sub};
use std::string::ToString;

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

impl Balance {
    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn from_str(value: &str, balance_type: BalanceType) -> Result<Balance, ParseIntError> {
        Ok(Balance {
            value: u256::from_str_radix(value, 10)?,
            balance_type,
        })
    }

    pub fn deserialize(data: &[u8], balance_type: BalanceType) -> Result<Balance, ParseIntError> {
        Ok(Balance {
            value: u256::from_be_bytes(
                data.try_into().unwrap()),
            balance_type
        })
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Balance {
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

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_tests() {
        let b = Balance::from_str("10", BalanceType::HOPR).unwrap();
        assert_eq!("10".to_string(), b.to_string());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::primitives::{Balance, BalanceType};

    #[wasm_bindgen]
    pub fn balance_from_str(value: &str, balance_type: BalanceType) -> JsResult<Balance> {
        ok_or_jserr!(Balance::from_str(value, balance_type))
    }
}
