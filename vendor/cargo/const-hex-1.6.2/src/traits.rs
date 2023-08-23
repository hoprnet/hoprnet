//! Modified from `hex`.

#![allow(clippy::ptr_as_ptr, clippy::borrow_as_ptr, clippy::missing_errors_doc)]

use core::iter;

#[cfg(feature = "alloc")]
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    rc::Rc,
    sync::Arc,
    vec::Vec,
};

/// Encoding values as hex string.
///
/// This trait is implemented for all `T` which implement `AsRef<[u8]>`. This
/// includes `String`, `str`, `Vec<u8>` and `[u8]`.
///
/// *Note*: instead of using this trait, you might want to use [`encode`].
///
/// # Examples
///
/// ```
/// use const_hex::ToHex;
///
/// assert_eq!("Hello world!".encode_hex::<String>(), "48656c6c6f20776f726c6421");
/// ```
#[cfg_attr(feature = "alloc", doc = "\n[`encode`]: crate::encode")]
#[cfg_attr(not(feature = "alloc"), doc = "\n[`encode`]: crate::encode_to_slice")]
#[deprecated(note = "use `encode` or other specialized functions instead")]
pub trait ToHex {
    /// Encode the hex strict representing `self` into the result. Lower case
    /// letters are used (e.g. `f9b4ca`)
    fn encode_hex<T: iter::FromIterator<char>>(&self) -> T;

    /// Encode the hex strict representing `self` into the result. Upper case
    /// letters are used (e.g. `F9B4CA`)
    fn encode_hex_upper<T: iter::FromIterator<char>>(&self) -> T;
}

struct BytesToHexChars<'a> {
    inner: core::slice::Iter<'a, u8>,
    table: &'static [u8; 16],
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8], table: &'static [u8; 16]) -> BytesToHexChars<'a> {
        BytesToHexChars {
            inner: inner.iter(),
            table,
            next: None,
        }
    }
}

impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let (high, low) = crate::byte2hex(*byte, self.table);
                self.next = Some(low as char);
                high as char
            }),
        }
    }
}

#[inline]
fn encode_to_iter<T: iter::FromIterator<char>>(source: &[u8], table: &'static [u8; 16]) -> T {
    BytesToHexChars::new(source, table).collect()
}

#[allow(deprecated)]
impl<T: AsRef<[u8]>> ToHex for T {
    fn encode_hex<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(self.as_ref(), crate::HEX_CHARS_LOWER)
    }

    fn encode_hex_upper<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(self.as_ref(), crate::HEX_CHARS_UPPER)
    }
}

/// Types that can be decoded from a hex string.
///
/// This trait is implemented for `Vec<u8>` and small `u8`-arrays.
///
/// # Example
///
/// ```
/// use const_hex::FromHex;
///
/// let buffer = <[u8; 12]>::from_hex("48656c6c6f20776f726c6421")?;
/// assert_eq!(buffer, *b"Hello world!");
/// # Ok::<(), const_hex::FromHexError>(())
/// ```
pub trait FromHex: Sized {
    type Error;

    /// Creates an instance of type `Self` from the given hex string, or fails
    /// with a custom error type.
    ///
    /// Both, upper and lower case characters are valid and can even be
    /// mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

#[cfg(feature = "alloc")]
impl<U: FromHex> FromHex for Box<U> {
    type Error = U::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Box::new(FromHex::from_hex(hex.as_ref())?))
    }
}

#[cfg(feature = "alloc")]
impl<U> FromHex for Cow<'_, U>
where
    U: Clone + ToOwned,
    U::Owned: FromHex,
{
    type Error = <U::Owned as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Cow::Owned(FromHex::from_hex(hex.as_ref())?))
    }
}

#[cfg(feature = "alloc")]
impl<U: FromHex> FromHex for Rc<U> {
    type Error = U::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Rc::new(FromHex::from_hex(hex.as_ref())?))
    }
}

#[cfg(feature = "alloc")]
impl<U: FromHex> FromHex for Arc<U> {
    type Error = U::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Arc::new(FromHex::from_hex(hex.as_ref())?))
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        crate::decode(hex.as_ref())
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<i8> {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let vec = crate::decode(hex.as_ref())?;
        // SAFETY: transmuting `u8` to `i8` is safe.
        Ok(unsafe { core::mem::transmute::<Vec<u8>, Vec<i8>>(vec) })
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Box<[u8]> {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        <Vec<u8>>::from_hex(hex).map(Vec::into_boxed_slice)
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Box<[i8]> {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        <Vec<i8>>::from_hex(hex).map(Vec::into_boxed_slice)
    }
}

impl<const N: usize> FromHex for [u8; N] {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut buf = [0u8; N];
        crate::decode_to_slice(hex.as_ref(), &mut buf)?;
        Ok(buf)
    }
}

impl<const N: usize> FromHex for [i8; N] {
    type Error = crate::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut buf = [0u8; N];
        crate::decode_to_slice(hex.as_ref(), &mut buf)?;
        // SAFETY: casting `[u8]` to `[i8]` is safe.
        Ok(unsafe { *(&buf as *const [u8; N] as *const [i8; N]) })
    }
}
