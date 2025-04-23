use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

use crate::errors::{GeneralError, GeneralError::InvalidInput, GeneralError::ParseError, Result};
use crate::prelude::BytesRepresentable;
use crate::traits::{IntoEndian, ToHex, UnitaryFloatOps};

pub type U256 = primitive_types::U256;

/// Represents an Ethereum address
#[derive(Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize, Hash, PartialOrd, Ord)]
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

    /// Turns the address into a checksum-ed address string
    /// according to https://eips.ethereum.org/EIPS/eip-55>
    pub fn to_checksum(&self) -> String {
        let address_hex = hex::encode(self.0);

        let hash = sha3::Keccak256::digest(address_hex.as_bytes());

        let mut ret = String::with_capacity(Self::SIZE * 2 + 2);
        ret.push_str("0x");

        for (i, c) in address_hex.chars().enumerate() {
            let nibble = (hash[i / 2] >> (((i + 1) % 2) * 4)) & 0xf;
            if nibble < 8 {
                ret.push(c);
            } else {
                ret.push(c.to_ascii_uppercase());
            }
        }
        ret
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("Address".into()))?))
    }
}

impl BytesRepresentable for Address {
    /// Fixed size of the address when encoded as bytes (e.g., via `as_ref()`).
    const SIZE: usize = 20;
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

impl From<alloy::primitives::Address> for Address {
    fn from(a: alloy::primitives::Address) -> Self {
        Address::from(a.0 .0)
    }
}
impl From<Address> for alloy::primitives::Address {
    fn from(a: Address) -> Self {
        alloy::primitives::Address::from_slice(a.as_ref())
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

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
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
        let regex = Regex::new(r"^\s*(\d+)\s*([A-z]+)\s*$").expect("should use valid regex pattern");
        let cap = regex.captures(s).ok_or(ParseError("Balance".into()))?;

        if cap.len() == 3 {
            Ok(Self::new_from_str(
                &cap[1],
                BalanceType::from_str(&cap[2]).map_err(|_| ParseError("Balance".into()))?,
            ))
        } else {
            Err(ParseError("Balance".into()))
        }
    }
}

/// Represents and Ethereum challenge.
/// This is a one-way encoding of the secp256k1 curve point to an Ethereum address.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct EthereumChallenge([u8; Self::SIZE]);

impl EthereumChallenge {
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);

        let mut ret = Self::default();
        ret.0.copy_from_slice(data);
        ret
    }
}

impl AsRef<[u8]> for EthereumChallenge {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for EthereumChallenge {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            value.try_into().map_err(|_| ParseError("EthereumChallenge".into()))?,
        ))
    }
}

impl BytesRepresentable for EthereumChallenge {
    const SIZE: usize = 20;
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

/// A type containing selected fields from  the `eth_getLogs` RPC calls.
///
/// This is further restricted to already mined blocks.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SerializableLog {
    /// Contract address
    pub address: Address,
    /// Topics
    pub topics: Vec<[u8; 32]>,
    /// Raw log data
    pub data: Vec<u8>,
    /// Transaction index
    pub tx_index: u64,
    /// Corresponding block number
    pub block_number: u64,
    /// Corresponding block hash
    pub block_hash: [u8; 32],
    /// Corresponding transaction hash
    pub tx_hash: [u8; 32],
    /// Log index
    pub log_index: u64,
    /// Removed flag
    pub removed: bool,
    /// Processed flag
    pub processed: Option<bool>,
    /// Processed time
    #[serde(with = "ts_seconds_option")]
    pub processed_at: Option<DateTime<Utc>>,
    /// Log hashes checksum
    pub checksum: Option<String>,
}

impl Display for SerializableLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "log #{} in tx #{} in block #{} of address {} with {} topics",
            self.log_index,
            self.tx_index,
            self.block_number,
            self.address,
            self.topics.len()
        )
    }
}

impl Ord for SerializableLog {
    fn cmp(&self, other: &Self) -> Ordering {
        let block_number_order = self.block_number.cmp(&other.block_number);
        if block_number_order == Ordering::Equal {
            let tx_index_order = self.tx_index.cmp(&other.tx_index);
            if tx_index_order == Ordering::Equal {
                self.log_index.cmp(&other.log_index)
            } else {
                tx_index_order
            }
        } else {
            block_number_order
        }
    }
}

impl PartialOrd<Self> for SerializableLog {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    fn address_tests() -> anyhow::Result<()> {
        let addr_1 = Address::try_from(hex!("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"))?;
        let addr_2 = Address::try_from(addr_1.as_ref())?;

        assert_eq!(addr_1, addr_2, "deserialized address does not match");
        assert_eq!(addr_1, Address::from_str("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")?);

        assert_eq!(addr_1, Address::from_str("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")?);

        assert_eq!(addr_1, Address::from_str(&addr_1.to_hex())?);

        Ok(())
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
    fn eth_challenge_tests() -> anyhow::Result<()> {
        let e_1 = EthereumChallenge::default();
        let e_2 = EthereumChallenge::try_from(e_1.as_ref())?;

        assert_eq!(e_1, e_2);

        Ok(())
    }

    #[test]
    fn u256_float_multiply() -> anyhow::Result<()> {
        assert_eq!(U256::one(), U256::one().mul_f64(1.0f64)?);
        assert_eq!(U256::one(), U256::from(10u64).mul_f64(0.1f64)?);

        // bad examples
        assert!(U256::one().mul_f64(-1.0).is_err());
        assert!(U256::one().mul_f64(1.1).is_err());

        Ok(())
    }

    #[test]
    fn u256_float_divide() -> anyhow::Result<()> {
        assert_eq!(U256::one(), U256::one().div_f64(1.0f64)?);

        assert_eq!(U256::from(2u64), U256::one().div_f64(0.5f64)?);
        assert_eq!(U256::from(10000u64), U256::one().div_f64(0.0001f64)?);

        // bad examples
        assert!(U256::one().div_f64(0.0).is_err());
        assert!(U256::one().div_f64(1.1).is_err());

        Ok(())
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

        let expected_be = hex!("0000000000000000000000000000000000000000000000000000001CBE991A08");
        assert_eq!(expected_be, be_bytes);
        assert_eq!(U256::from_be_bytes(expected_be), num);

        let expected_le = hex!("081A99BE1C000000000000000000000000000000000000000000000000000000");
        assert_eq!(expected_le, le_bytes);
        assert_eq!(U256::from_le_bytes(expected_le), num);
    }

    #[test]
    fn address_to_checksum_all_caps() -> anyhow::Result<()> {
        let addr_1 = Address::from_str("52908400098527886e0f7030069857d2e4169ee7")?;
        let value_1 = addr_1.to_checksum();
        let addr_2 = Address::from_str("8617e340b3d01fa5f11f306f4090fd50e238070d")?;
        let value_2 = addr_2.to_checksum();

        assert_eq!(
            value_1, "0x52908400098527886E0F7030069857D2E4169EE7",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
            "checksumed address does not match"
        );

        Ok(())
    }

    #[test]
    fn address_to_checksum_all_lower() -> anyhow::Result<()> {
        let addr_1 = Address::from_str("de709f2102306220921060314715629080e2fb77")?;
        let value_1 = addr_1.to_checksum();
        let addr_2 = Address::from_str("27b1fdb04752bbc536007a920d24acb045561c26")?;
        let value_2 = addr_2.to_checksum();

        assert_eq!(
            value_1, "0xde709f2102306220921060314715629080e2fb77",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x27b1fdb04752bbc536007a920d24acb045561c26",
            "checksumed address does not match"
        );

        Ok(())
    }

    #[test]
    fn address_to_checksum_all_normal() -> anyhow::Result<()> {
        let addr_1 = Address::from_str("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed")?;
        let addr_2 = Address::from_str("fb6916095ca1df60bb79ce92ce3ea74c37c5d359")?;
        let addr_3 = Address::from_str("dbf03b407c01e7cd3cbea99509d93f8dddc8c6fb")?;
        let addr_4 = Address::from_str("d1220a0cf47c7b9be7a2e6ba89f429762e7b9adb")?;

        let value_1 = addr_1.to_checksum();
        let value_2 = addr_2.to_checksum();
        let value_3 = addr_3.to_checksum();
        let value_4 = addr_4.to_checksum();

        assert_eq!(
            value_1, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "checksumed address does not match"
        );
        assert_eq!(
            value_3, "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "checksumed address does not match"
        );
        assert_eq!(
            value_4, "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
            "checksumed address does not match"
        );

        Ok(())
    }
}
