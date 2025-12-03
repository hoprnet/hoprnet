use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use sha3::Digest;

use crate::{
    errors::{
        GeneralError,
        GeneralError::{InvalidInput, ParseError},
        Result,
    },
    prelude::BytesRepresentable,
    traits::{IntoEndian, ToHex, UnitaryFloatOps},
};

pub type U256 = primitive_types::U256;

/// Represents an Ethereum address
#[derive(Clone, Copy, Eq, PartialEq, Default, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// according to [EIP-55](https://eips.ethereum.org/EIPS/eip-55).
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
    /// Fixed the size of the address when encoded as bytes (e.g., via `as_ref()`).
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

/// Represents and Ethereum challenge.
///
/// This is a one-way encoding of the secp256k1 curve point to an Ethereum address.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EthereumChallenge(pub Address);
impl AsRef<[u8]> for EthereumChallenge {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
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
    const SIZE: usize = Address::SIZE;
}

impl IntoEndian<32> for U256 {
    fn from_be_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        U256::from_big_endian(bytes.as_ref())
    }

    fn from_le_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        U256::from_little_endian(bytes.as_ref())
    }

    fn to_le_bytes(self) -> [u8; 32] {
        self.to_little_endian()
    }

    fn to_be_bytes(self) -> [u8; 32] {
        self.to_big_endian()
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

/// A type containing selected fields from the `eth_getLogs` RPC calls.
///
/// This is further restricted to already mined blocks.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    #[cfg_attr(feature = "serde", serde(with = "chrono::serde::ts_seconds_option"))]
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

/// Identifier of public keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyIdent<const N: usize = 4>(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; N]);

impl<const N: usize> Display for KeyIdent<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<u32> for KeyIdent<4> {
    fn from(value: u32) -> Self {
        Self(value.to_be_bytes())
    }
}

impl From<KeyIdent<4>> for u32 {
    fn from(value: KeyIdent<4>) -> Self {
        u32::from_be_bytes(value.0)
    }
}

impl From<u64> for KeyIdent<8> {
    fn from(value: u64) -> Self {
        Self(value.to_be_bytes())
    }
}

impl From<KeyIdent<8>> for u64 {
    fn from(value: KeyIdent<8>) -> Self {
        u64::from_be_bytes(value.0)
    }
}

impl<const N: usize> TryFrom<&[u8]> for KeyIdent<N> {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("KeyIdent".into()))?))
    }
}

impl<const N: usize> AsRef<[u8]> for KeyIdent<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Default for KeyIdent<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> BytesRepresentable for KeyIdent<N> {
    const SIZE: usize = N;
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use primitive_types::U256;

    use super::*;

    #[test]
    fn address_tests() -> anyhow::Result<()> {
        let addr_1 = Address::from(hex!("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"));
        let addr_2 = Address::try_from(addr_1.as_ref())?;

        assert_eq!(addr_1, addr_2, "deserialized address does not match");
        assert_eq!(addr_1, Address::from_str("Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")?);

        assert_eq!(addr_1, Address::from_str("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9")?);

        assert_eq!(addr_1, Address::from_str(&addr_1.to_hex())?);

        Ok(())
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
