use std::cmp::Ordering;
use ethnum::{u256, AsU256};
use std::ops::{Add, Sub};
use std::string::ToString;
use crate::errors::{Result, GeneralError::ParseError};
use crate::errors::GeneralError::MathError;

/// Represents an Ethereum address
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Address {
    addr: [u8; Self::SIZE],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Address {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE, "invalid length");
        let mut ret = Address {
            addr: [0u8; Self::SIZE]
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

impl Address {
    /// Size of the address when serialized
    pub const SIZE: usize = 20;
    
    pub fn from_str(value: &str) -> Result<Address> {
        Self::deserialize(&hex::decode(value).map_err(|_| ParseError)?)
    }

    pub fn deserialize(value: &[u8]) -> Result<Address> {
        if value.len() == Self::SIZE {
            let mut ret = Address{
                addr: [0u8; Self::SIZE]
            };
            ret.addr.copy_from_slice(&value);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}

/// Represents a type of the balance: native or HOPR tokens.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum BalanceType {
    Native,
    HOPR
}

/// Represents balance of some coin or token.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Balance {
    value: u256,
    balance_type: BalanceType
}

impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
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

    /// Retrieves the type (symbol) of the balance
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

    /// Serializes just the value of the balance (not the symbol)
    pub fn serialize_value(&self) -> Box<[u8]> {
        self.value().to_be_bytes().into()
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

/// Represents and Ethereum challenge.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct EthereumChallenge {
    challenge: [u8; Self::SIZE]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl EthereumChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);

        let mut ret = EthereumChallenge {
            challenge: [0u8; Self::SIZE]
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
    /// Size of the challenge when serialized
    pub const SIZE: usize = 20;
    
    pub fn deserialize(data: &[u8]) -> Result<EthereumChallenge> {
        if data.len() == Self::SIZE {
            Ok(EthereumChallenge::new(data))
        } else {
            Err(ParseError)
        }
    }
}

/// Represents the Ethereum's basic numeric type - unsigned 256-bit integer
#[derive(Clone, Eq, PartialEq, Debug)]
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
    fn address_tests() {
        let addr_1 = Address::new(&[0u8; Self::SIZE]);
        let addr_2 = Address::deserialize(&addr_1.serialize()).unwrap();

        assert_eq!(addr_1, addr_2);
    }

    #[test]
    fn balance_tests() {
        let b_1 = Balance::from_str("10", BalanceType::HOPR);
        assert_eq!("10".to_string(), b_1.to_string());

        let b_2 = Balance::deserialize(&b_1.serialize_value(), BalanceType::HOPR).unwrap();
        assert_eq!(b_1, b_2);
    }

    #[test]
    fn eth_challenge_tests() {
        let e_1 = EthereumChallenge::new(&[0u8; Self::SIZE]);
        let e_2 = EthereumChallenge::deserialize(&e_1.serialize()).unwrap();

        assert_eq!(e_1, e_2);
    }

    #[test]
    fn u256_tests() {
        let u_1 = U256::new("1234567899876543210");
        let u_2 = U256::deserialize(&u_1.serialize()).unwrap();

        assert_eq!(u_1, u_2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

    #[wasm_bindgen]
    impl Address {
        pub fn deserialize_address(data: &[u8]) -> JsResult<Address> {
            ok_or_jserr!(Address::deserialize(data))
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

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

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
