use ethnum::{u256, AsU256};
use getrandom::getrandom;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Shl, Shr, Sub};

use crate::errors::{GeneralError, GeneralError::InvalidInput, GeneralError::ParseError, Result};
use crate::traits::{AutoBinarySerializable, BinarySerializable, ToHex};

/// Represents an Ethereum address
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Address {
    addr: [u8; Self::SIZE],
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
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
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(js_name = "to_string"))]
    pub fn _to_string(&self) -> String {
        self.to_hex()
    }
    // }

    /// Creates a random Ethereum address, mostly used for testing
    pub fn random() -> Self {
        let mut addr = [0u8; Self::SIZE];
        getrandom(&mut addr[..]).unwrap();

        Self { addr }
    }
}

impl BinarySerializable for Address {
    const SIZE: usize = 20;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Address {
                addr: [0u8; Self::SIZE],
            };
            ret.addr.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.addr.into()
    }
}

impl TryFrom<[u8; Address::SIZE]> for Address {
    type Error = GeneralError;

    fn try_from(value: [u8; Address::SIZE]) -> std::result::Result<Self, Self::Error> {
        Address::from_bytes(&value)
    }
}

impl TryFrom<H160> for Address {
    type Error = GeneralError;

    fn try_from(value: H160) -> std::result::Result<Self, Self::Error> {
        Address::try_from(value.0)
    }
}

impl std::str::FromStr for Address {
    type Err = GeneralError;

    fn from_str(value: &str) -> Result<Address> {
        let decoded = if value.starts_with("0x") || value.starts_with("0X") {
            hex::decode(&value[2..])
        } else {
            hex::decode(value)
        }
        .map_err(|_| ParseError)?;
        if decoded.len() == Self::SIZE {
            let mut res = Self {
                addr: [0u8; Self::SIZE],
            };
            res.addr.copy_from_slice(&decoded);
            Ok(res)
        } else {
            Err(ParseError)
        }
    }
}

/// Represents a type of the balance: native or HOPR tokens.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum BalanceType {
    Native,
    HOPR,
}

/// Represents balance of some coin or token.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Balance {
    value: U256,
    balance_type: BalanceType,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Balance {
    /// Creates new balance of the given type from the base 10 integer string
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn from_str(value: &str, balance_type: BalanceType) -> Self {
        Self {
            value: U256 {
                value: u256::from_str_radix(value, 10).unwrap_or_else(|_| panic!("invalid number {}", value)),
            },
            balance_type,
        }
    }

    /// Creates zero balance of the given type
    pub fn zero(balance_type: BalanceType) -> Self {
        Self {
            value: U256::zero(),
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

    /// Serializes just the value of the balance (not the symbol)
    pub fn serialize_value(&self) -> Box<[u8]> {
        self.value().value().to_be_bytes().into()
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

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: U256 {
                value: self.value().value().add(other.value().value()),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Self {
            value: U256 {
                value: self.value().value().add(amount.as_u256()),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: U256 {
                value: self
                    .value()
                    .value()
                    .checked_sub(*other.value().value())
                    .unwrap_or(u256::ZERO),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Self {
            value: U256 {
                value: self.value().value().checked_sub(amount.as_u256()).unwrap_or(u256::ZERO),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn mul(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Self {
            value: U256 {
                value: self.value().value().mul(other.value().value()),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn imul(&self, amount: u64) -> Self {
        Self {
            value: U256 {
                value: self.value().value().mul(amount.as_u256()),
            },
            balance_type: self.balance_type,
        }
    }

    pub fn amount(&self) -> U256 {
        self.value
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.value(), self.balance_type)
    }
}

impl Balance {
    /// Size of the balance value is equal to U256 size (32 bytes)
    pub const SIZE: usize = U256::SIZE;

    pub fn new(value: U256, balance_type: BalanceType) -> Self {
        Balance { value, balance_type }
    }

    pub fn value(&self) -> &U256 {
        &self.value
    }

    pub fn deserialize(data: &[u8], balance_type: BalanceType) -> Result<Balance> {
        Ok(Balance {
            value: U256 {
                value: u256::from_be_bytes(data.try_into().map_err(|_| ParseError)?),
            },
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

impl BinarySerializable for EthereumChallenge {
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
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Snapshot {
    pub block_number: U256,
    pub transaction_index: U256,
    pub log_index: U256,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            block_number: U256::zero(),
            transaction_index: U256::zero(),
            log_index: U256::zero(),
        }
    }
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

impl BinarySerializable for Snapshot {
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct U256 {
    value: u256,
}

impl U256 {
    pub fn as_u128(&self) -> u128 {
        self.value.as_u128()
    }

    /// Multiply with float in the interval [0.0, 1.0]
    pub fn multiply_f64(&self, rhs: f64) -> Result<Self> {
        if rhs < 0.0 || rhs > 1.0 {
            return Err(GeneralError::InvalidInput);
        }

        if rhs == 1.0 {
            // special case: mantisse extraction does not work here
            Ok(Self {
                value: self.value.to_owned(),
            })
        } else if rhs == 0.0 {
            // special case: prevent from potential underflow errors
            Ok(U256::zero())
        } else {
            Ok((*self * U256::from((rhs + 1.0 + f64::EPSILON).to_bits() & 0x000fffffffffffffu64)) >> U256::from(52u64))
        }
    }

    /// Divide by float in the interval (0.0, 1.0]
    pub fn divide_f64(&self, rhs: f64) -> Result<Self> {
        if rhs <= 0.0 || rhs > 1.0 {
            return Err(GeneralError::InvalidInput);
        }

        if rhs == 1.0 {
            Ok(Self {
                value: self.value().to_owned(),
            })
        } else {
            let nom = *self << U256::from(52u64);
            let denom = U256::from((rhs + 1.0).to_bits() & 0x000fffffffffffffu64);

            Ok(nom / denom)
        }
    }
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
        Self { value: u256::ZERO }
    }

    pub fn max() -> Self {
        Self { value: u256::MAX }
    }

    pub fn one() -> Self {
        Self { value: u256::ONE }
    }

    pub fn as_u32(&self) -> u32 {
        self.value.as_u32()
    }

    pub fn as_u64(&self) -> u64 {
        self.value.as_u64()
    }

    pub fn addn(&self, n: u32) -> Self {
        Self {
            value: self.value + n as u128,
        }
    }

    pub fn muln(&self, n: u32) -> Self {
        Self {
            value: self.value * n as u128,
        }
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Mul for U256 {
    type Output = U256;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.mul(rhs.value),
        }
    }
}

impl Div for U256 {
    type Output = U256;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.div(rhs.value),
        }
    }
}

impl Shr for U256 {
    type Output = U256;

    fn shr(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.shr(rhs.value),
        }
    }
}

impl Shl for U256 {
    type Output = U256;

    fn shl(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.shl(rhs.value),
        }
    }
}

impl Add for U256 {
    type Output = U256;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.add(rhs.value),
        }
    }
}

impl Sub for U256 {
    type Output = U256;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.sub(rhs.value),
        }
    }
}

impl BinarySerializable for U256 {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(Self {
            value: u256::from_be_bytes(data.try_into().map_err(|_| ParseError)?),
        })
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.value.to_be_bytes().into()
    }
}

impl From<u256> for U256 {
    fn from(value: u256) -> Self {
        Self { value }
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        Self {
            value: u256::from(value),
        }
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        Self {
            value: u256::from(value),
        }
    }
}

impl From<u32> for U256 {
    fn from(value: u32) -> Self {
        Self {
            value: u256::from(value),
        }
    }
}

impl From<u16> for U256 {
    fn from(value: u16) -> Self {
        Self {
            value: u256::from(value),
        }
    }
}

impl From<u8> for U256 {
    fn from(value: u8) -> Self {
        Self {
            value: u256::from(value),
        }
    }
}

impl AsU256 for U256 {
    fn as_u256(self) -> ethnum::U256 {
        self.value
    }
}

impl U256 {
    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn from_inverse_probability(inverse_prob: U256) -> Result<U256> {
        let highest_prob = u256::MAX;
        if inverse_prob.value.gt(&u256::ZERO) {
            Ok(Self {
                value: highest_prob / inverse_prob.value,
            })
        } else if inverse_prob.value.eq(&u256::ZERO) {
            Ok(Self { value: highest_prob })
        } else {
            Err(InvalidInput)
        }
    }
}

// TODO: move this somewhere more appropriate
/// Represents an immutable authorization token used by the REST API.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct AuthorizationToken {
    id: String,
    token: Box<[u8]>,
}

impl AutoBinarySerializable for AuthorizationToken {}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AuthorizationToken {
    /// Creates new token from the serialized data and id
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(id: String, data: &[u8]) -> Self {
        assert!(data.len() < 2048, "invalid token size");
        Self { id, token: data.into() }
    }

    /// Gets the id of the token
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Gets token binary data
    pub fn get(&self) -> Box<[u8]> {
        self.token.clone()
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use std::cmp::Ordering;
    use std::str::FromStr;

    #[test]
    fn address_tests() {
        let addr_1 = Address::from_bytes(&hex!("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")).unwrap();
        let addr_2 = Address::from_bytes(&addr_1.to_bytes()).unwrap();

        assert_eq!(addr_1, addr_2, "deserialized address does not match");
        assert_eq!(
            addr_1,
            Address::from_str("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9").unwrap()
        );

        assert_eq!(
            addr_1,
            Address::from_str("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9").unwrap()
        );

        assert_eq!(addr_1, Address::from_str(&addr_1.to_hex()).unwrap());
    }

    #[test]
    fn balance_test_serialize() {
        let b_1 = Balance::from_str("10", BalanceType::HOPR);
        assert_eq!("10 HOPR".to_string(), b_1.to_string(), "to_string failed");

        let b_2 = Balance::deserialize(&b_1.serialize_value(), BalanceType::HOPR).unwrap();
        assert_eq!(b_1, b_2, "deserialized balance does not match");
    }

    #[test]
    fn balance_test_arithmetic() {
        let test_1 = 100_u32;
        let test_2 = 200_u32;

        let b3 = Balance::new(test_1.into(), BalanceType::HOPR);
        let b4 = Balance::new(test_2.into(), BalanceType::HOPR);

        assert_eq!(test_1 + test_2, b3.add(&b4).value().value().as_u32(), "add test failed");
        assert_eq!(test_2 - test_1, b4.sub(&b3).value().value().as_u32(), "sub test failed");
        assert_eq!(test_2 - 10, b4.isub(10).value().value().as_u32(), "sub test failed");

        assert_eq!(0_u32, b3.sub(&b4).value().value().as_u32(), "negative test failed");
        assert_eq!(
            0_u32,
            b3.isub(test_2 as u64).value().value().as_u32(),
            "negative test failed"
        );

        assert_eq!(
            test_1 * test_2,
            b3.mul(&b4).value().value.as_u32(),
            "multiplication test failed"
        );
        assert_eq!(
            test_2 * test_1,
            b4.mul(&b3).value().value.as_u32(),
            "multiplication test failed"
        );
        assert_eq!(
            test_2 * test_1,
            b4.imul(test_1 as u64).value().value.as_u32(),
            "multiplication test failed"
        );

        assert!(b3.lt(&b4) && b4.gt(&b3), "lte or lt test failed");
        assert!(b3.lte(&b3) && b4.gte(&b4), "gte or gt test failed");
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

        let u_3 = U256::new("2");
        let u_4 = U256::new("3");
        assert_eq!(Ordering::Less, u_3.cmp(&u_4));
        assert_eq!(Ordering::Greater, u_4.cmp(&u_3));
    }

    #[test]
    fn u256_float_multiply() {
        assert_eq!(U256::one(), U256::one().multiply_f64(1.0f64).unwrap());
        assert_eq!(U256::one(), U256::from(10u64).multiply_f64(0.1f64).unwrap());

        // bad examples
        assert!(U256::one().multiply_f64(-1.0).is_err());
        assert!(U256::one().multiply_f64(1.1).is_err());
    }

    #[test]
    fn u256_float_divide() {
        assert_eq!(U256::one(), U256::one().divide_f64(1.0f64).unwrap());

        assert_eq!(U256::from(2u64), U256::one().divide_f64(0.5f64).unwrap());
        assert_eq!(U256::from(10000u64), U256::one().divide_f64(0.0001f64).unwrap());

        // bad examples
        assert!(U256::one().divide_f64(0.0).is_err());
        assert!(U256::one().divide_f64(1.1).is_err());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256};
    use crate::traits::{BinarySerializable, ToHex};
    use std::cmp::Ordering;
    use std::str::FromStr;
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
        pub fn _deserialize(data: &[u8]) -> JsResult<Address> {
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
            *self
        }

        #[wasm_bindgen]
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

        #[wasm_bindgen]
        pub fn to_formatted_string(&self) -> String {
            self.to_string()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Balance) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            *self
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.value.to_string()
        }

        #[wasm_bindgen]
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

        #[wasm_bindgen]
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
            *self
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl U256 {
        #[wasm_bindgen(js_name = "from")]
        pub fn _from(value: u32) -> U256 {
            value.into()
        }

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
            ok_or_jserr!(U256::from_inverse_probability(*inverse_prob))
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_string()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &U256) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "cmp")]
        pub fn _cmp(&self, other: &U256) -> i32 {
            match self.cmp(other) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            *self
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Snapshot {
        #[wasm_bindgen(js_name = "default")]
        pub fn _default() -> Self {
            Snapshot::default()
        }
    }
}
