use ethnum::{u256, AsU256};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};
use std::string::ToString;

use crate::errors::{GeneralError::InvalidInput, GeneralError::ParseError, Result};
use crate::traits::{BinarySerializable, ToHex};

/// Represents an Ethereum address
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Address {
    addr: [u8; Self::SIZE],
}

impl Default for Address {
    fn default() -> Self {
        Self {
            addr: [0u8; Self::SIZE],
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Address {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE, "invalid length");
        let mut ret = Self::default();
        ret.addr.copy_from_slice(bytes);
        ret
    }

    pub fn to_bytes32(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(12 + Self::SIZE);
        ret.extend_from_slice(&[0u8; 12]);
        ret.extend_from_slice(&self.addr);
        ret.into_boxed_slice()
    }

    // impl std::string::ToString {
    pub fn to_string(&self) -> String {
        self.to_hex()
    }
    // }
}

impl BinarySerializable<'_> for Address {
    const SIZE: usize = 20;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Address {
                addr: [0u8; Self::SIZE],
            };
            ret.addr.copy_from_slice(&data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.addr.into()
    }
}

impl Address {
    // impl std::str::FromStr for Address {
    pub fn from_str(value: &str) -> Result<Address> {
        Self::from_bytes(&hex::decode(value).map_err(|_| ParseError)?)
    }
    // }
}

/// Represents a type of the balance: native or HOPR tokens.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum BalanceType {
    Native,
    HOPR,
}

/// Represents balance of some coin or token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Balance {
    value: u256,
    balance_type: BalanceType,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Balance {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn from_str(value: &str, balance_type: BalanceType) -> Self {
        Self {
            value: u256::from_str_radix(value, 10).unwrap(),
            balance_type,
        }
    }

    pub fn zero(balance_type: BalanceType) -> Self {
        Self {
            value: u256::ZERO,
            balance_type,
        }
    }

    /// Retrieves the type (symbol) of the balance
    pub fn balance_type(&self) -> BalanceType {
        self.balance_type
    }

    /// Creates balance of the given value with the same symbol
    pub fn of_same(&self, value: &str) -> Self {
        Self::from_str(value, self.balance_type)
    }

    // impl ToHex for Balance {
    pub fn to_hex(&self) -> String {
        hex::encode(self.value().to_be_bytes())
    }
    // }

    // impl std::string::ToString for Balance {
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
    // }

    /// Serializes just the value of the balance (not the symbol)
    pub fn serialize_value(&self) -> Box<[u8]> {
        self.value().to_be_bytes().into()
    }

    // impl PartialOrd for Balance {
    // NOTE: That these implementation rather panic to avoid comparison of different tokens
    // If PartialOrd was implemented, it would silently allow comparison of different tokens.

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

    // }

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: self.value().add(other.value()),
            balance_type: self.balance_type,
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Self {
            value: self.value().add(amount.as_u256()),
            balance_type: self.balance_type,
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: self.value().sub(other.value()),
            balance_type: self.balance_type,
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Self {
            value: self.value().sub(amount.as_u256()),
            balance_type: self.balance_type,
        }
    }

    pub fn mul(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: self.value().mul(other.value()),
            balance_type: self.balance_type,
        }
    }

    pub fn imul(&self, amount: u64) -> Self {
        Self {
            value: self.value().mul(amount.as_u256()),
            balance_type: self.balance_type,
        }
    }

    pub fn amount(&self) -> U256 {
        self.value.into()
    }
}

impl Balance {
    /// Size of the balance value is equal to U256 size (32 bytes)
    pub const SIZE: usize = U256::SIZE;

    pub fn new(value: u256, balance_type: BalanceType) -> Self {
        Balance { value, balance_type }
    }

    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn deserialize(data: &[u8], balance_type: BalanceType) -> Result<Balance> {
        Ok(Balance {
            value: u256::from_be_bytes(data.try_into().map_err(|_| ParseError)?),
            balance_type,
        })
    }
}

/// Represents and Ethereum challenge.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct EthereumChallenge {
    challenge: [u8; Self::SIZE],
}

impl Default for EthereumChallenge {
    fn default() -> Self {
        Self {
            challenge: [0u8; Self::SIZE],
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl EthereumChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);

        let mut ret = Self::default();
        ret.challenge.copy_from_slice(data);
        ret
    }
}

impl BinarySerializable<'_> for EthereumChallenge {
    const SIZE: usize = 20;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            Ok(EthereumChallenge::new(data))
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.challenge.into()
    }
}

/// Represents a snapshot in the blockchain
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Snapshot {
    pub block_number: U256,
    pub transaction_index: U256,
    pub log_index: U256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Snapshot {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(block_number: U256, transaction_index: U256, log_index: U256) -> Self {
        Self {
            block_number,
            transaction_index,
            log_index,
        }
    }
}

impl BinarySerializable<'_> for Snapshot {
    const SIZE: usize = 3 * U256::SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            Ok(Self {
                block_number: U256::from_bytes(&data[0..U256::SIZE])?,
                transaction_index: U256::from_bytes(&data[U256::SIZE..2 * U256::SIZE])?,
                log_index: U256::from_bytes(&data[2 * U256::SIZE..3 * U256::SIZE])?,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);

        ret.extend_from_slice(&self.block_number.to_bytes());
        ret.extend_from_slice(&self.transaction_index.to_bytes());
        ret.extend_from_slice(&self.log_index.to_bytes());
        ret.into_boxed_slice()
    }
}

/// Represents the Ethereum's basic numeric type - unsigned 256-bit integer
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct U256 {
    value: u256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl U256 {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(value: &str) -> Self {
        U256 {
            value: u256::from_str_radix(value, 10).expect("invalid decimal number string"),
        }
    }

    pub fn zero() -> Self {
        U256 { value: u256::ZERO }
    }

    pub fn one() -> Self {
        U256 { value: u256::ONE }
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn as_u32(&self) -> u32 {
        self.value.as_u32()
    }

    pub fn as_u64(&self) -> u64 {
        self.value.as_u64()
    }
}

impl BinarySerializable<'_> for U256 {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(U256 {
            value: u256::from_be_bytes(data.try_into().map_err(|_| ParseError)?),
        })
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.value.to_be_bytes().into()
    }
}

impl From<u256> for U256 {
    fn from(value: u256) -> Self {
        U256 { value }
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        U256 {
            value: u256::from(value),
        }
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        U256 {
            value: u256::from(value),
        }
    }
}

impl From<u32> for U256 {
    fn from(value: u32) -> Self {
        U256 {
            value: u256::from(value),
        }
    }
}

impl U256 {
    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn from_inverse_probability(inverse_prob: &u256) -> Result<U256> {
        let highest_prob = u256::MAX;
        if inverse_prob.gt(&u256::ZERO) {
            Ok(U256 {
                value: highest_prob / inverse_prob,
            })
        } else if inverse_prob.eq(&u256::ZERO) {
            Ok(U256 { value: highest_prob })
        } else {
            Err(InvalidInput)
        }
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    #[test]
    fn address_tests() {
        let addr_1 = Address::from_bytes(&hex!("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")).unwrap();
        let addr_2 = Address::from_bytes(&addr_1.to_bytes()).unwrap();

        assert_eq!(addr_1, addr_2, "deserialized address does not match");
        assert_eq!(addr_1, Address::from_str("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9").unwrap());
    }

    #[test]
    fn balance_tests() {
        let b_1 = Balance::from_str("10", BalanceType::HOPR);
        assert_eq!("10".to_string(), b_1.to_string(), "to_string failed");

        let b_2 = Balance::deserialize(&b_1.serialize_value(), BalanceType::HOPR).unwrap();
        assert_eq!(b_1, b_2, "deserialized balance does not match");

        let b3 = Balance::new(100_u32.into(), BalanceType::HOPR);
        let b4 = Balance::new(200_u32.into(), BalanceType::HOPR);

        assert_eq!(300_u32, b3.add(&b4).value().as_u32(), "add test failed");
        assert_eq!(100_u32, b4.sub(&b3).value().as_u32(), "sub test failed");

        assert!(b3.lt(&b4) && b4.gt(&b3), "lte or lt test failed");
        assert!(b3.lte(&b3) && b4.gte(&b4), "gte or gt test failed");

        //assert!(Balance::new(100_u32.into()).lte(), "lte or lte test failed")
    }

    #[test]
    fn eth_challenge_tests() {
        let e_1 = EthereumChallenge::default();
        let e_2 = EthereumChallenge::from_bytes(&e_1.to_bytes()).unwrap();

        assert_eq!(e_1, e_2);
    }

    #[test]
    fn snapshot_tests() {
        let s1 = Snapshot::new(1234_u32.into(), 4567_u32.into(), 102030_u32.into());
        let s2 = Snapshot::from_bytes(&s1.to_bytes()).unwrap();

        assert_eq!(s1, s2);
    }

    #[test]
    fn u256_tests() {
        let u_1 = U256::new("1234567899876543210");
        let u_2 = U256::from_bytes(&u_1.to_bytes()).unwrap();

        assert_eq!(u_1, u_2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256};
    use crate::traits::{BinarySerializable, ToHex};
    use ethnum::u256;
    use std::cmp::Ordering;
    use std::ops::{Add, Div};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl Address {
        #[wasm_bindgen(js_name = "from_string")]
        pub fn _from_str(str: &str) -> JsResult<Address> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_address(data: &[u8]) -> JsResult<Address> {
            ok_or_jserr!(Address::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Address) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Balance {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8], balance_type: BalanceType) -> JsResult<Balance> {
            ok_or_jserr!(Balance::deserialize(data, balance_type))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Balance) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "to_formatted_string")]
        pub fn _to_formatted_string(&self) -> String {
            format!("{} {:?}", self.value.div(&u256::from(10u16).pow(18)), self.balance_type)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl EthereumChallenge {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_challenge(data: &[u8]) -> JsResult<EthereumChallenge> {
            ok_or_jserr!(EthereumChallenge::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &EthereumChallenge) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Snapshot {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Snapshot> {
            ok_or_jserr!(Snapshot::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl U256 {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<U256> {
            ok_or_jserr!(U256::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "from_inverse_probability")]
        pub fn _from_inverse_probability(inverse_prob: &U256) -> JsResult<U256> {
            ok_or_jserr!(U256::from_inverse_probability(inverse_prob.value()))
        }

        pub fn addn(&self, amount: u32) -> Self {
            Self {
                value: self.value().add(u256::from(amount)),
            }
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &U256) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "cmp")]
        pub fn _cmp(&self, other: &U256) -> i32 {
            match self.cmp(&other) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
