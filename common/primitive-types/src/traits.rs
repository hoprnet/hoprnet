use crate::errors::GeneralError::ParseError;
use crate::errors::{GeneralError, Result};

/// A generic type that can be converted to a hexadecimal string.
pub trait ToHex: Sized {
    /// Hexadecimal representation of this type.
    fn to_hex(&self) -> String;

    /// Tries to parse the type from the hexadecimal representation.
    fn from_hex(str: &str) -> Result<Self>;
}

/// A type that can be serialized and deserialized to a binary form.
///
/// Implementing this trait automatically implements ToHex trait
/// which then uses the serialize method.
pub trait BinarySerializable: Sized {
    /// Minimum size of this type in bytes.
    const SIZE: usize;

    /// Deserializes the type from a binary blob.
    fn from_bytes(data: &[u8]) -> Result<Self>;

    /// Serializes the type into a fixed size binary blob.
    fn to_bytes(&self) -> Box<[u8]>;
}

/*pub trait FixedBytesEncodable<const N: usize>: AsRef<[u8; N]> + TryFrom<[u8; N], Error = GeneralError> {}

impl<const N: usize, T: FixedBytesEncodable<N>> ToHex for T {
    fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.as_ref()))
    }

    fn from_hex(str: &str) -> Result<Self> {
        if !str.is_empty() && str.len() % 2 == 0 {
            let data = if &str[..2] == "0x" || &str[..2] == "0X" {
                &str[2..]
            } else {
                str
            };

            let decoded: [u8; N] = hex::decode(data)
                .map_err(|_| ParseError)
                .and_then(|bytes| bytes.try_into().map_err(|_| ParseError))?;

            decoded.try_into()
        } else {
            Err(ParseError)
        }
    }
}*/

pub trait VariableBytesEncodable: AsRef<[u8]> + for <'a> TryFrom<&'a [u8], Error = GeneralError> {
    const SIZE: usize;
}

impl<T: VariableBytesEncodable> ToHex for T {
    fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.as_ref()))
    }

    fn from_hex(str: &str) -> Result<Self> {
        if !str.is_empty() && str.len() % 2 == 0 {
            let data = if &str[..2] == "0x" || &str[..2] == "0X" {
                &str[2..]
            } else {
                str
            };

            hex::decode(data)
                .map_err(|_| ParseError)
                .and_then(|bytes| T::try_from(&bytes))
        } else {
            Err(ParseError)
        }
    }
}

/// Allows type to be multiplied and divided by a float in range [0.0, 1.0].
pub trait UnitaryFloatOps: Sized {
    /// Multiply with float in the interval [0.0, 1.0]
    fn mul_f64(&self, rhs: f64) -> Result<Self>;
    /// Divide by float in the interval (0.0, 1.0]
    fn div_f64(&self, rhs: f64) -> Result<Self>;
}

/// Extension trait for fixed size numbers to allow conversion to/from endian representations.
pub trait IntoEndian<const N: usize> {
    /// Create instance from Big Endian bytes. Should panic if size is more than `N`.
    fn from_be_bytes<T: AsRef<[u8]>>(bytes: T) -> Self;
    /// Create instance from Little Endian bytes. Should panic if size is more than `N`.
    fn from_le_bytes<T: AsRef<[u8]>>(bytes: T) -> Self;
    /// Convert instance to Little Endian bytes.
    fn to_le_bytes(self) -> [u8; N];
    /// Convert instance to Big Endian bytes.
    fn to_be_bytes(self) -> [u8; N];
}

/// A trait that adds extension method to represent a time object as `Duration` since Unix epoch.
pub trait AsUnixTimestamp {
    /// Represents self as `Duration` since Unix epoch.
    fn as_unix_timestamp(&self) -> std::time::Duration;
}

impl AsUnixTimestamp for std::time::SystemTime {
    fn as_unix_timestamp(&self) -> std::time::Duration {
        self.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    }
}
