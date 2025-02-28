//! ByteSize is an utility that easily makes bytes size representation
//! and helps its arithmetic operations.
//!
//! ## Example
//!
//! ```ignore
//! extern crate bytesize;
//!
//! use bytesize::ByteSize;
//!
//! fn byte_arithmetic_operator() {
//!   let x = ByteSize::mb(1);
//!   let y = ByteSize::kb(100);
//!
//!   let plus = x + y;
//!   print!("{} bytes", plus.as_u64());
//!
//!   let minus = ByteSize::tb(100) - ByteSize::gb(4);
//!   print!("{} bytes", minus.as_u64());
//! }
//! ```
//!
//! It also provides its human readable string as follows:
//!
//! ```ignore=
//!  assert_eq!("482 GiB".to_string(), ByteSize::gb(518).to_string(true));
//!  assert_eq!("518 GB".to_string(), ByteSize::gb(518).to_string(false));
//! ```

mod parse;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "serde")]
use std::convert::TryFrom;

use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign};

/// byte size for 1 byte
pub const B: u64 = 1;
/// bytes size for 1 kilobyte
pub const KB: u64 = 1_000;
/// bytes size for 1 megabyte
pub const MB: u64 = 1_000_000;
/// bytes size for 1 gigabyte
pub const GB: u64 = 1_000_000_000;
/// bytes size for 1 terabyte
pub const TB: u64 = 1_000_000_000_000;
/// bytes size for 1 petabyte
pub const PB: u64 = 1_000_000_000_000_000;

/// bytes size for 1 kibibyte
pub const KIB: u64 = 1_024;
/// bytes size for 1 mebibyte
pub const MIB: u64 = 1_048_576;
/// bytes size for 1 gibibyte
pub const GIB: u64 = 1_073_741_824;
/// bytes size for 1 tebibyte
pub const TIB: u64 = 1_099_511_627_776;
/// bytes size for 1 pebibyte
pub const PIB: u64 = 1_125_899_906_842_624;

static UNITS: &str = "KMGTPE";
static UNITS_SI: &str = "kMGTPE";
static LN_KB: f64 = 6.931471806; // ln 1024
static LN_KIB: f64 = 6.907755279; // ln 1000

pub fn kb<V: Into<u64>>(size: V) -> u64 {
    size.into() * KB
}

pub fn kib<V: Into<u64>>(size: V) -> u64 {
    size.into() * KIB
}

pub fn mb<V: Into<u64>>(size: V) -> u64 {
    size.into() * MB
}

pub fn mib<V: Into<u64>>(size: V) -> u64 {
    size.into() * MIB
}

pub fn gb<V: Into<u64>>(size: V) -> u64 {
    size.into() * GB
}

pub fn gib<V: Into<u64>>(size: V) -> u64 {
    size.into() * GIB
}

pub fn tb<V: Into<u64>>(size: V) -> u64 {
    size.into() * TB
}

pub fn tib<V: Into<u64>>(size: V) -> u64 {
    size.into() * TIB
}

pub fn pb<V: Into<u64>>(size: V) -> u64 {
    size.into() * PB
}

pub fn pib<V: Into<u64>>(size: V) -> u64 {
    size.into() * PIB
}

/// Byte size representation
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct ByteSize(pub u64);

impl ByteSize {
    #[inline(always)]
    pub const fn b(size: u64) -> ByteSize {
        ByteSize(size)
    }

    #[inline(always)]
    pub const fn kb(size: u64) -> ByteSize {
        ByteSize(size * KB)
    }

    #[inline(always)]
    pub const fn kib(size: u64) -> ByteSize {
        ByteSize(size * KIB)
    }

    #[inline(always)]
    pub const fn mb(size: u64) -> ByteSize {
        ByteSize(size * MB)
    }

    #[inline(always)]
    pub const fn mib(size: u64) -> ByteSize {
        ByteSize(size * MIB)
    }

    #[inline(always)]
    pub const fn gb(size: u64) -> ByteSize {
        ByteSize(size * GB)
    }

    #[inline(always)]
    pub const fn gib(size: u64) -> ByteSize {
        ByteSize(size * GIB)
    }

    #[inline(always)]
    pub const fn tb(size: u64) -> ByteSize {
        ByteSize(size * TB)
    }

    #[inline(always)]
    pub const fn tib(size: u64) -> ByteSize {
        ByteSize(size * TIB)
    }

    #[inline(always)]
    pub const fn pb(size: u64) -> ByteSize {
        ByteSize(size * PB)
    }

    #[inline(always)]
    pub const fn pib(size: u64) -> ByteSize {
        ByteSize(size * PIB)
    }

    #[inline(always)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn to_string_as(&self, si_unit: bool) -> String {
        to_string(self.0, si_unit)
    }
}

pub fn to_string(bytes: u64, si_prefix: bool) -> String {
    let unit = if si_prefix { KIB } else { KB };
    let unit_base = if si_prefix { LN_KIB } else { LN_KB };
    let unit_prefix = if si_prefix {
        UNITS_SI.as_bytes()
    } else {
        UNITS.as_bytes()
    };
    let unit_suffix = if si_prefix { "iB" } else { "B" };

    if bytes < unit {
        format!("{} B", bytes)
    } else {
        let size = bytes as f64;
        let exp = match (size.ln() / unit_base) as usize {
            e if e == 0 => 1,
            e => e,
        };

        format!(
            "{:.1} {}{}",
            (size / unit.pow(exp as u32) as f64),
            unit_prefix[exp - 1] as char,
            unit_suffix
        )
    }
}

impl Display for ByteSize {
    fn fmt(&self, f: &mut Formatter) ->fmt::Result {
        f.pad(&to_string(self.0, false))
    }
}

impl Debug for ByteSize {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

macro_rules! commutative_op {
    ($t:ty) => {
        impl Add<ByteSize> for $t {
            type Output = ByteSize;
            #[inline(always)]
            fn add(self, rhs: ByteSize) -> ByteSize {
                ByteSize(rhs.0 + (self as u64))
            }
        }

        impl Mul<ByteSize> for $t {
            type Output = ByteSize;
            #[inline(always)]
            fn mul(self, rhs: ByteSize) -> ByteSize {
                ByteSize(rhs.0 * (self as u64))
            }
        }
    };
}

commutative_op!(u64);
commutative_op!(u32);
commutative_op!(u16);
commutative_op!(u8);

impl Add<ByteSize> for ByteSize {
    type Output = ByteSize;

    #[inline(always)]
    fn add(self, rhs: ByteSize) -> ByteSize {
        ByteSize(self.0 + rhs.0)
    }
}

impl AddAssign<ByteSize> for ByteSize {
    #[inline(always)]
    fn add_assign(&mut self, rhs: ByteSize) {
        self.0 += rhs.0
    }
}

impl<T> Add<T> for ByteSize
    where T: Into<u64> {
    type Output = ByteSize;
    #[inline(always)]
    fn add(self, rhs: T) -> ByteSize {
        ByteSize(self.0 + (rhs.into() as u64))
    }
}

impl<T> AddAssign<T> for ByteSize
    where T: Into<u64> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs.into() as u64;
    }
}

impl<T> Mul<T> for ByteSize
    where T: Into<u64> {
    type Output = ByteSize;
    #[inline(always)]
    fn mul(self, rhs: T) -> ByteSize {
        ByteSize(self.0 * (rhs.into() as u64))
    }
}

impl<T> MulAssign<T> for ByteSize
    where T: Into<u64> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs.into() as u64;
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ByteSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ByteSizeVistor;

        impl<'de> de::Visitor<'de> for ByteSizeVistor {
            type Value = ByteSize;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an integer or string")
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                if let Ok(val) = u64::try_from(value) {
                    Ok(ByteSize(val))
                } else {
                    Err(E::invalid_value(
                        de::Unexpected::Signed(value),
                        &"integer overflow",
                    ))
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(ByteSize(value))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                if let Ok(val) = value.parse() {
                    Ok(val)
                } else {
                    Err(E::invalid_value(
                        de::Unexpected::Str(value),
                        &"parsable string",
                    ))
                }
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(ByteSizeVistor)
        } else {
            deserializer.deserialize_u64(ByteSizeVistor)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for ByteSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            <str>::serialize(self.to_string().as_str(), serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_op() {
        let mut x = ByteSize::mb(1);
        let y = ByteSize::kb(100);

        assert_eq!((x + y).as_u64(), 1_100_000u64);

        assert_eq!((x + (100 * 1000) as u64).as_u64(), 1_100_000);

        assert_eq!((x * 2u64).as_u64(), 2_000_000);

        x += y;
        assert_eq!(x.as_u64(), 1_100_000);
        x *= 2u64;
        assert_eq!(x.as_u64(), 2_200_000);
    }

    #[test]
    fn test_arithmetic_primitives() {
        let mut x = ByteSize::mb(1);

        assert_eq!((x + MB as u64).as_u64(), 2_000_000);

        assert_eq!((x + MB as u32).as_u64(), 2_000_000);

        assert_eq!((x + KB as u16).as_u64(), 1_001_000);

        assert_eq!((x + B as u8).as_u64(), 1_000_001);

        x += MB as u64;
        x += MB as u32;
        x += 10u16;
        x += 1u8;
        assert_eq!(x.as_u64(), 3_000_011);
    }

    #[test]
    fn test_comparison() {
        assert!(ByteSize::mb(1) == ByteSize::kb(1000));
        assert!(ByteSize::mib(1) == ByteSize::kib(1024));
        assert!(ByteSize::mb(1) != ByteSize::kib(1024));
        assert!(ByteSize::mb(1) < ByteSize::kib(1024));
        assert!(ByteSize::b(0) < ByteSize::tib(1));
    }

    fn assert_display(expected: &str, b: ByteSize) {
        assert_eq!(expected, format!("{}", b));
    }

    #[test]
    fn test_display() {
        assert_display("215 B", ByteSize::b(215));
        assert_display("1.0 KB", ByteSize::kb(1));
        assert_display("301.0 KB", ByteSize::kb(301));
        assert_display("419.0 MB", ByteSize::mb(419));
        assert_display("518.0 GB", ByteSize::gb(518));
        assert_display("815.0 TB", ByteSize::tb(815));
        assert_display("609.0 PB", ByteSize::pb(609));
    }

    #[test]
    fn test_display_alignment() {
        assert_eq!("|357 B     |", format!("|{:10}|", ByteSize(357)));
        assert_eq!("|     357 B|", format!("|{:>10}|", ByteSize(357)));
        assert_eq!("|357 B     |", format!("|{:<10}|", ByteSize(357)));
        assert_eq!("|  357 B   |", format!("|{:^10}|", ByteSize(357)));

        assert_eq!("|-----357 B|", format!("|{:->10}|", ByteSize(357)));
        assert_eq!("|357 B-----|", format!("|{:-<10}|", ByteSize(357)));
        assert_eq!("|--357 B---|", format!("|{:-^10}|", ByteSize(357)));
    }

    fn assert_to_string(expected: &str, b: ByteSize, si: bool) {
        assert_eq!(expected.to_string(), b.to_string_as(si));
    }

    #[test]
    fn test_to_string_as() {
        assert_to_string("215 B", ByteSize::b(215), true);
        assert_to_string("215 B", ByteSize::b(215), false);

        assert_to_string("1.0 kiB", ByteSize::kib(1), true);
        assert_to_string("1.0 KB", ByteSize::kib(1), false);

        assert_to_string("293.9 kiB", ByteSize::kb(301), true);
        assert_to_string("301.0 KB", ByteSize::kb(301), false);

        assert_to_string("1.0 MiB", ByteSize::mib(1), true);
        assert_to_string("1048.6 KB", ByteSize::mib(1), false);

        // a bug case: https://github.com/flang-project/bytesize/issues/8
        assert_to_string("1.9 GiB", ByteSize::mib(1907), true);
        assert_to_string("2.0 GB", ByteSize::mib(1908), false);

        assert_to_string("399.6 MiB", ByteSize::mb(419), true);
        assert_to_string("419.0 MB", ByteSize::mb(419), false);

        assert_to_string("482.4 GiB", ByteSize::gb(518), true);
        assert_to_string("518.0 GB", ByteSize::gb(518), false);

        assert_to_string("741.2 TiB", ByteSize::tb(815), true);
        assert_to_string("815.0 TB", ByteSize::tb(815), false);

        assert_to_string("540.9 PiB", ByteSize::pb(609), true);
        assert_to_string("609.0 PB", ByteSize::pb(609), false);
    }

    #[test]
    fn test_default() {
        assert_eq!(ByteSize::b(0), ByteSize::default());
    }

    #[test]
    fn test_to_string() {
        assert_to_string("609.0 PB", ByteSize::pb(609), false);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        #[derive(Serialize, Deserialize)]
        struct S {
            x: ByteSize,
        }

        let s: S = serde_json::from_str(r#"{ "x": "5 B" }"#).unwrap();
        assert_eq!(s.x, ByteSize(5));

        let s: S = serde_json::from_str(r#"{ "x": 1048576 }"#).unwrap();
        assert_eq!(s.x, "1 MiB".parse::<ByteSize>().unwrap());

        let s: S = toml::from_str(r#"x = "2.5 MiB""#).unwrap();
        assert_eq!(s.x, "2.5 MiB".parse::<ByteSize>().unwrap());

        // i64 MAX
        let s: S = toml::from_str(r#"x = "9223372036854775807""#).unwrap();
        assert_eq!(s.x, "9223372036854775807".parse::<ByteSize>().unwrap());
    }
}
