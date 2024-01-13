//! Root module for 256-bit signed integer type.

mod api;
mod cmp;
mod convert;
mod fmt;
mod iter;
mod ops;
mod parse;

pub use self::convert::AsI256;
use crate::uint::U256;
use core::num::ParseIntError;

/// A 256-bit signed integer type.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct I256(pub [i128; 2]);

impl I256 {
    /// The additive identity for this integer type, i.e. `0`.
    pub const ZERO: Self = I256([0; 2]);

    /// The multiplicative identity for this integer type, i.e. `1`.
    pub const ONE: Self = I256::new(1);

    /// The multiplicative inverse for this integer type, i.e. `-1`.
    pub const MINUS_ONE: Self = I256::new(-1);

    /// Creates a new 256-bit integer value from a primitive `i128` integer.
    #[inline]
    pub const fn new(value: i128) -> Self {
        I256::from_words(value >> 127, value)
    }

    /// Creates a new 256-bit integer value from high and low words.
    #[inline]
    pub const fn from_words(hi: i128, lo: i128) -> Self {
        #[cfg(target_endian = "little")]
        {
            I256([lo, hi])
        }
        #[cfg(target_endian = "big")]
        {
            I256([hi, lo])
        }
    }

    /// Splits a 256-bit integer into high and low words.
    #[inline]
    pub const fn into_words(self) -> (i128, i128) {
        #[cfg(target_endian = "little")]
        {
            let I256([lo, hi]) = self;
            (hi, lo)
        }
        #[cfg(target_endian = "big")]
        {
            let I256([hi, lo]) = self;
            (hi, lo)
        }
    }

    /// Get the low 128-bit word for this signed integer.
    #[inline]
    pub fn low(&self) -> &i128 {
        #[cfg(target_endian = "little")]
        {
            &self.0[0]
        }
        #[cfg(target_endian = "big")]
        {
            &self.0[1]
        }
    }

    /// Get the low 128-bit word for this signed integer as a mutable reference.
    #[inline]
    pub fn low_mut(&mut self) -> &mut i128 {
        #[cfg(target_endian = "little")]
        {
            &mut self.0[0]
        }
        #[cfg(target_endian = "big")]
        {
            &mut self.0[1]
        }
    }

    /// Get the high 128-bit word for this signed integer.
    #[inline]
    pub fn high(&self) -> &i128 {
        #[cfg(target_endian = "little")]
        {
            &self.0[1]
        }
        #[cfg(target_endian = "big")]
        {
            &self.0[0]
        }
    }

    /// Get the high 128-bit word for this signed integer as a mutable
    /// reference.
    #[inline]
    pub fn high_mut(&mut self) -> &mut i128 {
        #[cfg(target_endian = "little")]
        {
            &mut self.0[1]
        }
        #[cfg(target_endian = "big")]
        {
            &mut self.0[0]
        }
    }

    /// Converts a prefixed string slice in base 16 to an integer.
    ///
    /// The string is expected to be an optional `+` or `-` sign followed by
    /// the `0x` prefix and finally the digits. Leading and trailing whitespace
    /// represent an error.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use ethnum::I256;
    /// assert_eq!(I256::from_str_hex("0x2A"), Ok(I256::new(42)));
    /// assert_eq!(I256::from_str_hex("-0xa"), Ok(I256::new(-10)));
    /// ```
    pub fn from_str_hex(src: &str) -> Result<Self, ParseIntError> {
        crate::parse::from_str_radix(src, 16, Some("0x"))
    }

    /// Converts a prefixed string slice in a base determined by the prefix to
    /// an integer.
    ///
    /// The string is expected to be an optional `+` or `-` sign followed by
    /// the one of the supported prefixes and finally the digits. Leading and
    /// trailing whitespace represent an error. The base is determined based
    /// on the prefix:
    ///
    /// * `0b`: base `2`
    /// * `0o`: base `8`
    /// * `0x`: base `16`
    /// * no prefix: base `10`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use ethnum::I256;
    /// assert_eq!(I256::from_str_prefixed("-0b101"), Ok(I256::new(-0b101)));
    /// assert_eq!(I256::from_str_prefixed("0o17"), Ok(I256::new(0o17)));
    /// assert_eq!(I256::from_str_prefixed("-0xa"), Ok(I256::new(-0xa)));
    /// assert_eq!(I256::from_str_prefixed("42"), Ok(I256::new(42)));
    /// ```
    pub fn from_str_prefixed(src: &str) -> Result<Self, ParseIntError> {
        crate::parse::from_str_prefixed(src)
    }

    /// Same as [`I256::from_str_prefixed`] but as a `const fn`. This method is
    /// not intended to be used directly but rather through the [`crate::int`]
    /// macro.
    #[doc(hidden)]
    pub const fn const_from_str_prefixed(src: &str) -> Self {
        parse::const_from_str_prefixed(src)
    }

    /// Cast to a primitive `i8`.
    #[inline]
    pub const fn as_i8(self) -> i8 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i16`.
    #[inline]
    pub const fn as_i16(self) -> i16 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i32`.
    #[inline]
    pub const fn as_i32(self) -> i32 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i64`.
    #[inline]
    pub const fn as_i64(self) -> i64 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i128`.
    #[inline]
    pub const fn as_i128(self) -> i128 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u8`.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u16`.
    #[inline]
    pub const fn as_u16(self) -> u16 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u32`.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u128`.
    #[inline]
    pub const fn as_u128(self) -> u128 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a `U256`.
    #[inline]
    pub const fn as_u256(self) -> U256 {
        let Self([a, b]) = self;
        U256([a as _, b as _])
    }

    /// Cast to a primitive `isize`.
    #[inline]
    pub const fn as_isize(self) -> isize {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `usize`.
    #[inline]
    pub const fn as_usize(self) -> usize {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `f32`.
    #[inline]
    pub fn as_f32(self) -> f32 {
        self.as_f64() as _
    }

    /// Cast to a primitive `f64`.
    #[inline]
    pub fn as_f64(self) -> f64 {
        let sign = self.signum128() as f64;
        self.unsigned_abs().as_f64() * sign
    }
}

#[cfg(test)]
mod tests {
    use crate::I256;

    #[test]
    #[allow(clippy::float_cmp)]
    fn converts_to_f64() {
        assert_eq!((-I256::from_words(1, 0)).as_f64(), -(2.0f64.powi(128)))
    }
}
