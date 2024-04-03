use crate::errors::GeneralError::ParseError;
use crate::errors::{GeneralError, Result};

/// A generic type that can be converted to a hexadecimal string.
pub trait ToHex {
    /// Hexadecimal representation of this type.
    fn to_hex(&self) -> String;

    /// Tries to parse the type from the hexadecimal representation.
    fn from_hex(str: &str) -> Result<Self>
    where
        Self: Sized;
}

/// Represents a type that can be encoded to/decoded from a fixed sized byte array of size `N`.
/// This requires processing and memory allocation in order to represent the type in binary encoding.
///
/// Differences between [BytesEncodable] and [BytesRepresentable] :
/// - [BytesRepresentable] is already internally carrying the encoded representation of the type,
/// so no additional encoding or allocation is required to represent the type as a byte array.
/// - [BytesEncodable] requires additional transformation and allocation in order to represent the type as a fixed size
/// byte array.
/// - [BytesEncodable] is the strict superset of [BytesRepresentable]: meaning the former can be possibly implemented
/// for a type that already implements the latter, but it is not possible vice-versa.
pub trait BytesEncodable<const N: usize>: Into<[u8; N]> + for<'a> TryFrom<&'a [u8], Error = GeneralError> {
    /// Size of the encoded byte array. Defaults to `N` and should not be overridden.
    const SIZE: usize = N;

    /// Convenience function to represent the
    /// A shorthand for `let v: [u8; N] = self.into()`.
    #[inline]
    fn into_encoded(self) -> [u8; N] {
        self.into()
    }

    /// Convenience function to encode the type into a Box.
    #[inline]
    fn into_boxed(self) -> Box<[u8]> {
        Box::new(self.into_encoded())
    }
}

/// Represents a type that is already internally represented by a fixed size byte array,
/// and therefore requires no memory allocation to represent the type in binary encoding.
///
/// This is a strict subset of [BytesEncodable], see its documentation for details.
pub trait BytesRepresentable: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = GeneralError> {
    /// Size of the encoded byte array.
    const SIZE: usize;

    /// Convenience function to copy this type's binary representation into a Box.
    #[inline]
    fn into_boxed(self) -> Box<[u8]> {
        self.as_ref().to_vec().into_boxed_slice()
    }
}

impl<T: BytesRepresentable> ToHex for T {
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
