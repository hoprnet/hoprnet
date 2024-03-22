use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

use crate::errors::{GeneralError, GeneralError::InvalidInput, GeneralError::ParseError, Result};
use crate::traits::{BinarySerializable, IntoEndian, ToHex, UnitaryFloatOps};

pub type U256 = primitive_types::U256;

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

    /// Checks if the address is all zeroes.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|e| 0_u8.eq(e))
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
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

impl From<Address> for [u8; Address::SIZE] {
    fn from(value: Address) -> Self {
        value.0
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
pub enum BalanceType {
    /// Native tokens of the underlying chain.
    Native,
    /// HOPR tokens.
    HOPR,
}

impl BalanceType {
    /// Creates [Balance] of 1 of this type.
    pub fn one(self) -> Balance {
        self.balance(1)
    }

    /// Creates zero [Balance] of this type.
    pub fn zero(self) -> Balance {
        self.balance(0)
    }

    /// Creates [Balance] of the given `amount` of this type.
    pub fn balance<T: Into<U256>>(self, amount: T) -> Balance {
        Balance::new(amount, self)
    }

    /// Deserializes the given amount and creates a new [Balance] instance.
    /// The bytes are assumed to be in Big Endian order.
    /// The method panics if more than 32 `bytes` were given.
    pub fn balance_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Balance {
        Balance::new(U256::from_be_bytes(bytes), self)
    }
}

/// Represents balance of some coin or token.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance(U256, BalanceType);

impl Balance {
    /// Number of digits in the base unit
    pub const SCALE: usize = 19;

    /// Creates a new balance given the value and type
    pub fn new<T: Into<U256>>(value: T, balance_type: BalanceType) -> Self {
        Self(value.into(), balance_type)
    }

    /// Creates new balance of the given type from the base 10 integer string
    pub fn new_from_str(value: &str, balance_type: BalanceType) -> Self {
        Self(
            U256::from_dec_str(value).unwrap_or_else(|_| panic!("invalid decimal number {value}")),
            balance_type,
        )
    }

    /// Creates zero balance of the given type
    pub fn zero(balance_type: BalanceType) -> Self {
        Self(U256::zero(), balance_type)
    }

    /// Retrieves the type (symbol) of the balance
    pub fn balance_type(&self) -> BalanceType {
        self.1
    }

    /// Creates balance of the given value with the same symbol
    pub fn of_same(&self, value: &str) -> Self {
        Self::new_from_str(value, self.1)
    }

    pub fn amount(&self) -> U256 {
        self.0
    }

    pub fn amount_base_units(&self) -> String {
        let val = self.0.to_string();

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
        format!("{} {}", self.amount_base_units(), self.1)
    }

    pub fn to_value_string(&self) -> String {
        self.0.to_string()
    }
}

impl<T: Into<U256>> From<(T, BalanceType)> for Balance {
    fn from(value: (T, BalanceType)) -> Self {
        Self(value.0.into(), value.1)
    }
}

impl PartialOrd for Balance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.eq(&other.1).then(|| self.0.partial_cmp(&other.0)).flatten()
    }
}

impl<T: Into<U256>> Add<T> for Balance {
    type Output = Balance;

    fn add(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_add(rhs.into()), self.1)
    }
}

impl Add for Balance {
    type Output = Balance;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs.0)
    }
}

impl Add<&Balance> for Balance {
    type Output = Balance;

    fn add(self, rhs: &Balance) -> Self::Output {
        self.add(rhs.0)
    }
}

impl<T: Into<U256>> Sub<T> for Balance {
    type Output = Balance;

    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_sub(rhs.into()), self.1)
    }
}

impl Sub for Balance {
    type Output = Balance;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs.0)
    }
}

impl Sub<&Balance> for Balance {
    type Output = Balance;

    fn sub(self, rhs: &Balance) -> Self::Output {
        self.sub(rhs.0)
    }
}

impl<T: Into<U256>> Mul<T> for Balance {
    type Output = Balance;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_mul(rhs.into()), self.1)
    }
}

impl Mul for Balance {
    type Output = Balance;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs.0)
    }
}

impl UnitaryFloatOps for Balance {
    fn mul_f64(&self, rhs: f64) -> Result<Self> {
        self.0.mul_f64(rhs).map(|v| Self(v, self.1))
    }

    fn div_f64(&self, rhs: f64) -> Result<Self> {
        self.0.div_f64(rhs).map(|v| Self(v, self.1))
    }
}

impl Debug for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as Display
        write!(f, "{} {}", self.0, self.1)
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl FromStr for Balance {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let regex = Regex::new(r"^\s*(\d+)\s*([A-z]+)\s*$").unwrap();
        let cap = regex.captures(s).ok_or(ParseError)?;

        if cap.len() == 3 {
            Ok(Self::new_from_str(
                &cap[1],
                BalanceType::from_str(&cap[2]).map_err(|_| ParseError)?,
            ))
        } else {
            Err(ParseError)
        }
    }
}

/// Represents and Ethereum challenge.
/// This is a one-way encoding of the secp256k1 curve point to an Ethereum address.
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

impl BinarySerializable for U256 {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() <= Self::SIZE {
            Ok(Self::from_big_endian(data))
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = [0u8; Self::SIZE];
        self.to_big_endian(&mut ret);
        ret.into()
    }
}

impl IntoEndian<32> for U256 {
    fn from_be_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        U256::from_big_endian(bytes.as_ref())
    }

    fn from_le_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        U256::from_little_endian(bytes.as_ref())
    }

    fn to_le_bytes(self) -> [u8; 32] {
        let mut ret = [0u8; 32];
        self.to_little_endian(&mut ret);
        ret
    }

    fn to_be_bytes(self) -> [u8; 32] {
        let mut ret = [0u8; 32];
        self.to_big_endian(&mut ret);
        ret
    }
}

impl UnitaryFloatOps for U256 {
    fn mul_f64(&self, rhs: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&rhs) {
            return Err(InvalidInput);
        }

        if rhs == 1.0 {
            // special case: mantissa extraction does not work here
            Ok(Self(self.0))
        } else if rhs == 0.0 {
            // special case: prevent from potential underflow errors
            Ok(U256::zero())
        } else {
            Ok(
                (*self * U256::from((rhs + 1.0 + f64::EPSILON).to_bits() & 0x000fffffffffffff_u64))
                    >> U256::from(52_u64),
            )
        }
    }

    fn div_f64(&self, rhs: f64) -> Result<Self> {
        if rhs <= 0.0 || rhs > 1.0 {
            return Err(InvalidInput);
        }

        if rhs == 1.0 {
            Ok(Self(self.0))
        } else {
            let nom = *self << U256::from(52_u64);
            let denom = U256::from((rhs + 1.0).to_bits() & 0x000fffffffffffff_u64);

            Ok(nom / denom)
        }
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use primitive_types::U256;
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
    fn balance_test_arithmetic() {
        let test_1 = 100_u32;
        let test_2 = 200_u32;

        let b3 = Balance::new(test_1, BalanceType::HOPR);
        let b4 = Balance::new(test_2, BalanceType::HOPR);

        assert_eq!(test_1 + test_2, b3.add(b4).amount().as_u32(), "add test failed");
        assert_eq!(test_2 - test_1, b4.sub(b3).amount().as_u32(), "sub test failed");
        assert_eq!(test_2 - 10, b4.sub(10).amount().as_u32(), "sub test failed");

        assert_eq!(0_u32, b3.sub(b4).amount().as_u32(), "negative test failed");
        assert_eq!(0_u32, b3.sub(test_2 as u64).amount().as_u32(), "negative test failed");

        assert_eq!(
            test_1 * test_2,
            b3.mul(b4).amount().as_u32(),
            "multiplication test failed"
        );
        assert_eq!(
            test_2 * test_1,
            b4.mul(b3).amount().as_u32(),
            "multiplication test failed"
        );
        assert_eq!(
            test_2 * test_1,
            b4.mul(test_1 as u64).amount().as_u32(),
            "multiplication test failed"
        );

        assert!(matches!(b3.partial_cmp(&b4), Some(Ordering::Less)));
        assert!(matches!(b4.partial_cmp(&b3), Some(Ordering::Greater)));

        assert!(matches!(b3.partial_cmp(&b3), Some(Ordering::Equal)));
        assert!(matches!(b4.partial_cmp(&b4), Some(Ordering::Equal)));

        let other: Balance = (b4.amount(), BalanceType::Native).into();
        assert!(other.partial_cmp(&b4).is_none());
    }

    #[test]
    fn balance_test_formatted_string() {
        let mut base = "123".to_string();
        for _ in 0..Balance::SCALE - 3 {
            base += "0";
        }

        let b1 = Balance::new_from_str(&base, BalanceType::HOPR);
        let b2 = b1.mul(100);
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
        let b2 = b1.mul(100);
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
    fn u256_float_multiply() {
        assert_eq!(U256::one(), U256::one().mul_f64(1.0f64).unwrap());
        assert_eq!(U256::one(), U256::from(10u64).mul_f64(0.1f64).unwrap());

        // bad examples
        assert!(U256::one().mul_f64(-1.0).is_err());
        assert!(U256::one().mul_f64(1.1).is_err());
    }

    #[test]
    fn u256_float_divide() {
        assert_eq!(U256::one(), U256::one().div_f64(1.0f64).unwrap());

        assert_eq!(U256::from(2u64), U256::one().div_f64(0.5f64).unwrap());
        assert_eq!(U256::from(10000u64), U256::one().div_f64(0.0001f64).unwrap());

        // bad examples
        assert!(U256::one().div_f64(0.0).is_err());
        assert!(U256::one().div_f64(1.1).is_err());
    }

    #[test]
    fn u256_endianness() {
        let num: U256 = 123456789000_u128.into();

        let be_bytes = num.to_be_bytes();
        let le_bytes = num.to_le_bytes();

        assert_ne!(
            be_bytes, le_bytes,
            "sanity check: input number must have different endianness"
        );
        assert_eq!(
            num.to_bytes().as_ref(),
            be_bytes.as_ref(),
            "to_bytes must yield big endian"
        );

        let expected_be = hex!("0000000000000000000000000000000000000000000000000000001CBE991A08");
        assert_eq!(expected_be, be_bytes);
        assert_eq!(U256::from_be_bytes(expected_be), num);

        let expected_le = hex!("081A99BE1C000000000000000000000000000000000000000000000000000000");
        assert_eq!(expected_le, le_bytes);
        assert_eq!(U256::from_le_bytes(expected_le), num);
    }
}
