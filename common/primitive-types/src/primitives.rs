use ethnum::{u256, AsU256};
use rand::rngs::OsRng;
use rand::RngCore;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Shl, Shr, Sub};
use std::str::FromStr;

use crate::errors::{GeneralError, GeneralError::InvalidInput, GeneralError::ParseError, Result};
use crate::traits::{AutoBinarySerializable, BinarySerializable, ToHex};

/// Represents an Ethereum address
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct Address([u8; Self::SIZE]);

impl Debug for Address {
    // Intentionally same as Display
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for Address {
    /// Defaults to all zeroes.
    fn default() -> Self {
        Self([0u8; Self::SIZE])
    }
}

impl Address {
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE, "invalid length");
        let mut ret = Self::default();
        ret.0.copy_from_slice(bytes);
        ret
    }

    pub fn to_bytes32(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(12 + Self::SIZE);
        ret.extend_from_slice(&[0u8; 12]);
        ret.extend_from_slice(&self.0);
        ret.into_boxed_slice()
    }

    /// Creates a random Ethereum address, mostly used for testing
    pub fn random() -> Self {
        // Uses getrandom, because it cannot bring in dependency on core-crypto
        let mut addr = [0u8; Self::SIZE];
        OsRng.fill_bytes(&mut addr[..]);
        Self(addr)
    }

    /// Checks if the address is all zeroes.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|e| 0_u8.eq(e))
    }
}

impl BinarySerializable for Address {
    const SIZE: usize = 20;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self([0u8; Self::SIZE]);
            ret.0.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.0.into()
    }
}

impl From<[u8; Address::SIZE]> for Address {
    fn from(value: [u8; Address::SIZE]) -> Self {
        Self(value)
    }
}

impl From<primitive_types::H160> for Address {
    fn from(value: primitive_types::H160) -> Self {
        Self(value.0)
    }
}

impl From<Address> for primitive_types::H160 {
    fn from(value: Address) -> Self {
        primitive_types::H160::from_slice(&value.0)
    }
}

impl FromStr for Address {
    type Err = GeneralError;

    fn from_str(value: &str) -> Result<Address> {
        Self::from_hex(value)
    }
}

/// Represents a type of the balance: native or HOPR tokens.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceType {
    Native,
    HOPR,
}

impl Display for BalanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native => write!(f, "Native"),
            Self::HOPR => write!(f, "HOPR"),
        }
    }
}

impl FromStr for BalanceType {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NATIVE" => Ok(Self::Native),
            "HOPR" => Ok(Self::HOPR),
            _ => Err(ParseError),
        }
    }
}

/// Represents balance of some coin or token.
#[derive(Clone, Copy, Eq, Serialize, Deserialize)]
pub struct Balance {
    value: U256,
    balance_type: BalanceType,
}

impl Balance {
    /// Creates new balance of the given type from the base 10 integer string
    pub fn new_from_str(value: &str, balance_type: BalanceType) -> Self {
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
        Self::new_from_str(value, self.balance_type)
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

    /// Divides the balance by a float in inverval (0,1]
    pub fn div_f64(&self, divisor: f64) -> Self {
        Self {
            value: self.value().divide_f64(divisor).expect("divisor must be in (0,1]"),
            balance_type: self.balance_type,
        }
    }

    /// Multiplies the balance by a float in inverval (0,1]
    pub fn mul_f64(&self, coefficient: f64) -> Self {
        Self {
            value: self
                .value()
                .multiply_f64(coefficient)
                .expect("coefficient must be in (0,1]"),
            balance_type: self.balance_type,
        }
    }

    pub fn amount(&self) -> U256 {
        self.value
    }

    pub fn amount_base_units(&self) -> String {
        let val = self.value.to_string();

        match val.len().cmp(&Self::SCALE) {
            Ordering::Greater => {
                let (l, r) = val.split_at(val.len() - Self::SCALE + 1);
                format!("{l}.{}", &r[..r.len() - (val.len() - Self::SCALE)],)
            }
            Ordering::Less => format!("0.{empty:0>width$}", empty = &val, width = Self::SCALE - 1,),
            Ordering::Equal => {
                let (l, r) = val.split_at(1);
                format!("{l}.{r}")
            }
        }
    }

    pub fn to_formatted_string(&self) -> String {
        format!("{} {}", self.amount_base_units(), self.balance_type)
    }

    pub fn to_value_string(&self) -> String {
        self.value.to_string()
    }
}

impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.balance_type == other.balance_type
    }
}

impl Debug for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as Display
        write!(f, "{} {:?}", self.value(), self.balance_type)
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.value(), self.balance_type)
    }
}

impl FromStr for Balance {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let regex = Regex::new(r"^\s*(\d+)\s*([A-z]+)\s*$").unwrap();
        let cap = regex.captures(s).ok_or(ParseError)?;

        if cap.len() == 3 {
            Ok(Self::new_from_str(&cap[1], BalanceType::from_str(&cap[2])?))
        } else {
            Err(ParseError)
        }
    }
}

impl Balance {
    /// Size of the balance value is equal to U256 size (32 bytes)
    pub const SIZE: usize = U256::SIZE;

    /// Number of digits in the base unit
    pub const SCALE: usize = 19;

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

impl EthereumChallenge {
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

impl Snapshot {
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
pub struct U256 {
    value: u256,
}

impl U256 {
    pub fn as_u128(&self) -> u128 {
        self.value.as_u128()
    }

    /// Multiply with float in the interval [0.0, 1.0]
    pub fn multiply_f64(&self, rhs: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&rhs) {
            return Err(InvalidInput);
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

    pub fn new(value: &str) -> Self {
        Self {
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
        self.add(n)
    }

    pub fn muln(&self, n: u32) -> Self {
        self.mul(n)
    }
}

impl Default for U256 {
    fn default() -> Self {
        Self::zero()
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Sum for U256 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self {
            value: iter.map(|u| u.value).sum(),
        }
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

impl Mul<u32> for U256 {
    type Output = U256;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            value: self.value * rhs as u128,
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

impl Add<u32> for U256 {
    type Output = U256;

    fn add(self, rhs: u32) -> Self::Output {
        Self {
            value: self.value + rhs as u128,
        }
    }
}

impl AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        self.value.add_assign(&rhs.value);
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

// TODO: should we change U256 to have underlying type primitive_types::U256 and ditch ethnum?

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

impl From<primitive_types::U256> for U256 {
    fn from(value: primitive_types::U256) -> Self {
        U256::from(&value)
    }
}

impl From<&primitive_types::U256> for U256 {
    fn from(value: &primitive_types::U256) -> Self {
        let mut tmp = [0u8; 32];
        value.to_big_endian(&mut tmp);

        Self {
            value: u256::from_be_bytes(tmp),
        }
    }
}

impl From<U256> for primitive_types::U256 {
    fn from(value: U256) -> Self {
        primitive_types::U256::from(&value)
    }
}

impl From<&U256> for primitive_types::U256 {
    fn from(value: &U256) -> Self {
        primitive_types::U256::from_big_endian(&value.value.to_be_bytes())
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
pub struct AuthorizationToken {
    id: String,
    token: Box<[u8]>,
}

impl AutoBinarySerializable for AuthorizationToken {}

impl AuthorizationToken {
    /// Creates new token from the serialized data and id
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
        let b_1 = Balance::new_from_str("10", BalanceType::HOPR);
        assert_eq!("10 HOPR".to_string(), b_1.to_string(), "to_string failed");

        let b_2 = Balance::deserialize(&b_1.serialize_value(), BalanceType::HOPR).unwrap();
        assert_eq!(b_1, b_2, "deserialized balance does not match");

        assert_eq!(
            b_1,
            Balance::from_str(&b_1.to_string()).expect("must parse balance 1"),
            "string representations must match 1"
        );
        assert_eq!(
            b_1,
            Balance::from_str("10HOPR").expect("must parse balance 2"),
            "string representations must match 2"
        );
        assert_eq!(
            b_1,
            Balance::from_str(" 10   hOpR").expect("must parse balance 3"),
            "string representations must match 3"
        );
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
    fn balance_test_formatted_string() {
        let mut base = "123".to_string();
        for _ in 0..Balance::SCALE - 3 {
            base += "0";
        }

        let b1 = Balance::new_from_str(&base, BalanceType::HOPR);
        let b2 = b1.imul(100);
        let b3 = Balance::new_from_str(&base[..Balance::SCALE - 3], BalanceType::HOPR);
        let b4 = Balance::new_from_str(&base[..Balance::SCALE - 1], BalanceType::HOPR);

        assert_eq!("1.230000000000000000 HOPR", b1.to_formatted_string());
        assert_eq!("123.0000000000000000 HOPR", b2.to_formatted_string());
        assert_eq!("0.001230000000000000 HOPR", b3.to_formatted_string());
        assert_eq!("0.123000000000000000 HOPR", b4.to_formatted_string());
    }

    #[test]
    fn balance_test_value_string() {
        let mut base = "123".to_string();
        for _ in 0..Balance::SCALE - 3 {
            base += "0";
        }

        let b1 = Balance::new_from_str(&base, BalanceType::HOPR);
        let b2 = b1.imul(100);
        let b3 = Balance::new_from_str(&base[..Balance::SCALE - 3], BalanceType::HOPR);
        let b4 = Balance::new_from_str(&base[..Balance::SCALE - 1], BalanceType::HOPR);

        assert_eq!("1230000000000000000", b1.to_value_string());
        assert_eq!("123000000000000000000", b2.to_value_string());
        assert_eq!("1230000000000000", b3.to_value_string());
        assert_eq!("123000000000000000", b4.to_value_string());
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

    #[test]
    fn u256_conversions() {
        let u256_ethereum =
            primitive_types::U256::from_str("ef35a3f4fda07a4719ed5960b40ac51e67f013c1c444662eaff3b3d217492957")
                .unwrap();

        assert_eq!(
            U256::from(u256_ethereum),
            U256::from_hex("ef35a3f4fda07a4719ed5960b40ac51e67f013c1c444662eaff3b3d217492957").unwrap()
        );

        let u256 = U256::from_hex("ef35a3f4fda07a4719ed5960b40ac51e67f013c1c444662eaff3b3d217492957").unwrap();

        assert_eq!(
            primitive_types::U256::from(u256),
            primitive_types::U256::from_str("ef35a3f4fda07a4719ed5960b40ac51e67f013c1c444662eaff3b3d217492957")
                .unwrap()
        );
    }
}
