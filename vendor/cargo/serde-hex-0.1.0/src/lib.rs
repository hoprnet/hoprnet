//! The `serde-hex` crate contains various utilities for Serialization/Deserialization
//! of hexadecimal values using [`serde`](https://crates.io/crates/serde).
//!
//! The core utility of this crate is the `SerHex` trait.  Once implemented, `SerHex`
//! allows for easy configuration of hexadecimal serialization/deserialization with
//! `serde-derive`:
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_hex;
//! use serde_hex::{SerHex,StrictPfx};
//!
//! #[derive(Debug,Serialize,Deserialize)]
//! struct Foo {
//!    #[serde(with = "SerHex::<StrictPfx>")]
//!    bar: [u8;32]
//! }
//!
//! # fn main() {}
//! ```
//!
//! The above example will cause serde to serialize `Bar` into a hexadecimal string
//! with strict sizing (padded with leading zeroes), and prefixing (leading `0x`).
//! The possible configurations allow for any combination of strict/compact
//! representations, prefixing, and capitalizing (e.g.; `Compact`,
//! `StrictCapPfx`, etc...).
//!
//! This crate provides implementations of `SerHex` for all unsigned integer types,
//! as well as generic impls for arrays of types which implement `SerHex`.  The generic
//! impls apply only to strict variants of the trait, and only for arrays of length 1
//! through 64 (no impl is provided for arrays of length 0 since there isn't really
//! a reasonable way to represent a zero-sized value in hex).
//!
//!
//!
#![warn(missing_docs)]

extern crate array_init;
extern crate serde;
extern crate smallvec;

#[macro_use]
pub mod macros;
pub mod config;
pub mod types;
pub mod utils;

pub use config::*;
pub use types::{Error, ParseHexError};

use serde::{Deserialize, Deserializer, Serializer};
use smallvec::SmallVec;
use std::iter::FromIterator;
use std::{error, io};

/// Trait specifying custom serialization and deserialization logic from a
/// hexadecimal string to some arbitrary type.  This trait can be used to apply
/// custom parsing when using serde's `#[derive(Serialize,Deserialize)]`
/// flag.  Just add `#[serde(with = "SerHex")]` above any fields which implement
/// this trait.  Simplistic default implimentations for the the `serialize` and
/// `deserialize` methods are provided based on `into_hex_raw` and `from_hex_raw` respectively.
pub trait SerHex<C>: Sized
where
    C: HexConf,
{
    /// Error type of the implementation.
    ///
    /// Unless you have a compelling reason to do so, it is best to use the error
    /// type exposed by `serde-hex`, since this is the error used for most provided
    /// implementations (the generic array impls will work with any error that
    /// implements [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
    /// for the `serde-hex` error type).
    type Error: error::Error;

    /// Attept to convert `self` to hexadecimal, writing the resultant bytes to some buffer.
    fn into_hex_raw<D>(&self, dst: D) -> Result<(), Self::Error>
    where
        D: io::Write;

    /// Attempt to parse some buffer of hexadecimal bytes into an instance of `Self`.
    fn from_hex_raw<S>(src: S) -> Result<Self, Self::Error>
    where
        S: AsRef<[u8]>;

    /// Attempt to convert `self` into a hexadecimal string representation.
    fn into_hex(&self) -> Result<String, Self::Error> {
        let mut dst: Vec<u8> = Vec::with_capacity(32);
        self.into_hex_raw(&mut dst)?;
        Ok(String::from_utf8(dst).expect("invalid UTF-8 bytes in parsing"))
    }

    /// Attempt to convert a slice of hexadecimal bytes into an instance of `Self`.
    fn from_hex<S>(src: S) -> Result<Self, Self::Error>
    where
        S: AsRef<[u8]>,
    {
        Self::from_hex_raw(src)
    }

    /// Attempt to serialize `self` into a hexadecimal string representation.
    ///
    /// *NOTE*: The default implementation attempts to avoid heap-allocation with a
    /// [`SmallVec`](https://docs.rs/smallvec/) of size `[u8;64]`. This default will
    /// prevent heap-alloc for non-prefixed serializations of `[u8;32]` or smaller.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;
        let mut dst = SmallVec::<[u8; 64]>::new();
        self.into_hex_raw(&mut dst).map_err(S::Error::custom)?;
        // if `dst` is not valid UTF-8 bytes, the underlying implementation
        // is very broken, and you should be ashamed of yourelf.
        debug_assert!(::std::str::from_utf8(dst.as_ref()).is_ok());
        let s = unsafe { ::std::str::from_utf8_unchecked(dst.as_ref()) };
        serializer.serialize_str(s)
    }

    /// Attempt to deserialize a hexadecimal string into an instance of `Self`.
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let buff: &[u8] = Deserialize::deserialize(deserializer)?;
        let rslt = Self::from_hex_raw(buff).map_err(D::Error::custom)?;
        Ok(rslt)
    }
}

/// Variant of `SerHex` for serializing/deserializing `Option` types.
///
/// Any type `T` which implements `SerHex<C>` implements `SerHexOpt<C>`
/// automatically.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_json;
/// # extern crate serde_hex;
/// # use serde_hex::{SerHexOpt,CompactPfx};
/// #
/// #[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
/// struct MaybeNum {
///     #[serde(with = "SerHexOpt::<CompactPfx>")]
///     num: Option<u64>
/// }
///
/// # fn main() {
/// let s: MaybeNum = serde_json::from_str(r#"{"num":"0xff"}"#).unwrap();
/// assert_eq!(s,MaybeNum { num: Some(255) });
///
/// let n: MaybeNum = serde_json::from_str(r#"{"num":null}"#).unwrap();
/// assert_eq!(n,MaybeNum { num: None });
/// # }
/// ```
///
pub trait SerHexOpt<C>: Sized + SerHex<C>
where
    C: HexConf,
{
    /// Same as `SerHex::serialize`, except for `Option<Self>` instead of `Self`.
    fn serialize<S>(option: &Option<Self>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;
        if let Some(ref src) = *option {
            let mut dst = SmallVec::<[u8; 64]>::new();
            Self::into_hex_raw(src, &mut dst).map_err(S::Error::custom)?;
            // if `dst` is not valid UTF-8 bytes, the underlying implementation
            // is very broken, and you should be ashamed of yourelf.
            debug_assert!(::std::str::from_utf8(dst.as_ref()).is_ok());
            let s = unsafe { ::std::str::from_utf8_unchecked(dst.as_ref()) };
            //serializer.serialize_str(s)
            serializer.serialize_some(s)
        } else {
            serializer.serialize_none()
        }
    }

    /// Same as `SerHex::deserialize`, except for `Option<Self>` instead of `Self`.
    fn deserialize<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let option: Option<&[u8]> = Deserialize::deserialize(deserializer)?;
        if let Some(ref buff) = option {
            let rslt = Self::from_hex_raw(buff).map_err(D::Error::custom)?;
            Ok(Some(rslt))
        } else {
            Ok(None)
        }
    }
}

impl<T, C> SerHexOpt<C> for T
where
    T: Sized + SerHex<C>,
    C: HexConf,
{
}

/// Variant of `SerHex` for serializing/deserializing sequence types as
/// contiguous hexadecimal strings.
///
/// *NOTE*: `Compact` configurations are not compatible with this trait.
/// The size of each element must be consistent in order to avoid ambiguous
/// encoding.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate serde_json;
/// # extern crate serde_hex;
/// # use serde_hex::{SerHexSeq,StrictPfx};
/// #
/// #[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
/// struct Bytes(#[serde(with = "SerHexSeq::<StrictPfx>")] Vec<u8>);
///
/// # fn main() {
/// let bytes: Bytes = serde_json::from_str(r#""0xdeadbeef""#).unwrap();
/// assert_eq!(bytes,Bytes(vec![0xde,0xad,0xbe,0xef]));
/// # }
/// ```
///
pub trait SerHexSeq<C>: Sized + SerHex<Strict> + SerHex<StrictCap>
where
    C: HexConf,
{
    /// expected size (in bytes) of a single element.  used to partition
    /// the hexadecimal string into individual elements.
    fn size() -> usize;

    /// Same as `SerHex::serialize`, but for sequences of `Self`.
    fn serialize<'a, S, T>(sequence: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = &'a Self>,
        Self: 'a,
    {
        use serde::ser::Error;
        let mut dst = SmallVec::<[u8; 128]>::new();
        if <C as HexConf>::withpfx() {
            dst.extend_from_slice(b"0x");
        }
        if <C as HexConf>::withcap() {
            for elem in sequence.into_iter() {
                <Self as SerHex<StrictCap>>::into_hex_raw(elem, &mut dst)
                    .map_err(S::Error::custom)?;
            }
        } else {
            for elem in sequence.into_iter() {
                <Self as SerHex<Strict>>::into_hex_raw(elem, &mut dst).map_err(S::Error::custom)?;
            }
        }
        let s = unsafe { ::std::str::from_utf8_unchecked(dst.as_ref()) };
        serializer.serialize_str(s)
    }

    /// Same as `SerHex::deserialize`, but for sequences of `Self`.
    fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<Self>,
    {
        use serde::de::Error;
        let raw: &[u8] = Deserialize::deserialize(deserializer)?;
        let src = if raw.starts_with(b"0x") {
            &raw[2..]
        } else {
            &raw[..]
        };
        let hexsize = Self::size() * 2;
        if src.len() % hexsize == 0 {
            let mut buff = Vec::with_capacity(src.len() / hexsize);
            for chunk in src.chunks(hexsize) {
                let elem =
                    <Self as SerHex<Strict>>::from_hex_raw(chunk).map_err(D::Error::custom)?;
                buff.push(elem);
            }
            Ok(buff.into_iter().collect())
        } else {
            Err(D::Error::custom("bad hexadecimal sequence size"))
        }
    }
}

impl_serhex_uint!(u8, 1);
impl_serhex_uint!(u16, 2);
impl_serhex_uint!(u32, 4);
impl_serhex_uint!(u64, 8);

// implement strict variants of `SerHex` for arrays of `T` with
// lengths of 1 through 64 (where `T` implements the strict variants
// of `SerHex` as well).
impl_serhex_strict_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64
);
