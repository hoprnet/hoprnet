#![deny(warnings, missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/string/0.2.1")]

//! A UTF-8 encoded string with configurable byte storage.
//!
//! This crate provides `String`, a type similar to its std counterpart, but
//! with one significant difference: the underlying byte storage is
//! configurable. In other words, `String<T>` is a marker type wrapping `T`,
//! indicating that it represents a UTF-8 encoded string.
//!
//! For example, one can represent small strings (stack allocated) by wrapping
//! an array:
//!
//! ```
//! # use string::*;
//! let s: String<[u8; 2]> = String::try_from([b'h', b'i']).unwrap();
//! assert_eq!(&s[..], "hi");
//! ```

#[cfg(feature = "bytes")]
extern crate bytes;

use std::{borrow, fmt, hash, ops, str};
use std::default::Default;

/// A UTF-8 encoded string with configurable byte storage.
///
/// This type differs from `std::String` in that it is generic over the
/// underlying byte storage, enabling it to use `Vec<[u8]>`, `&[u8]`, or third
/// party types, such as [`Bytes`].
///
/// In order to construct `String` via any of the non-unsafe constructors,
/// the backing storage needs to implement the `StableAsRef` marker trait.
/// If you wish to construct `String` with a type that does not implement `StableAsRef`,
/// you can use the `from_utf8_unchecked` constructor.
///
/// [`Bytes`]: https://docs.rs/bytes/0.4.8/bytes/struct.Bytes.html
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct String<T = Vec<u8>> {
    value: T,
}

impl<T> String<T> {
    /// Get a reference to the underlying byte storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # use string::*;
    /// let s = String::new();
    /// let vec = s.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the underlying byte storage.
    ///
    /// It is inadvisable to directly manipulate the byte storage. This function
    /// is unsafe as the bytes could no longer be valid UTF-8 after mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use string::*;
    /// let mut s = String::new();
    ///
    /// unsafe {
    ///     let vec = s.get_mut();
    /// }
    /// ```
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Unwraps this `String`, returning the underlying byte storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # use string::*;
    /// let s = String::new();
    /// let vec = s.into_inner();
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Creates a new `String` from a &str.
    ///
    /// Use `TryFrom` for conversion from &[u8].
    ///
    /// ```
    /// # use string::*;
    /// let _: String<Vec<u8>> = String::from_str("nice str");
    /// ```
    pub fn from_str<'a>(src: &'a str) -> String<T>
        where T: From<&'a [u8]> + StableAsRef,
    {
        let value: T = src.as_bytes().into();
        Self { value }
    }
}

impl String {
    /// Creates a new empty `String`.
    ///
    /// Given that the `String` is empty, this will not allocate.
    ///
    /// # Examples
    ///
    /// Basic usage
    ///
    /// ```
    /// let s = String::new();
    /// assert_eq!(s, "");
    /// ```
    pub fn new() -> String {
        String::default()
    }
}

impl<T> String<T>
    where T: AsRef<[u8]>,
{
    /// Converts the provided value to a `String` without checking that the
    /// given value is valid UTF-8.
    ///
    /// Use `TryFrom` for a safe conversion.
    ///
    /// # Safety
    ///
    /// You must ensure that:
    ///
    /// 1. The backing storage type `T` adheres to the contract as documented on the `StableAsRef`
    ///    marker trait.
    /// 2. If `T` implements `AsRef<[u8]>` and/or `AsMut<[u8]>`, the byte slice returned
    ///    by calling `as_ref` and/or `as_mut` on the provided value represents valid utf-8.
    pub unsafe fn from_utf8_unchecked(value: T) -> String<T> {
        String { value }
    }
}

impl<T> PartialEq<str> for String<T>
    where T: AsRef<[u8]>
{
    fn eq(&self, other: &str) -> bool {
        &self[..] == other
    }
}

impl<T> hash::Hash for String<T>
    where T: AsRef<[u8]>
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        ops::Deref::deref(self).hash(state);
    }
}

impl<T> ops::Deref for String<T>
    where T: AsRef<[u8]>
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        let b = self.value.as_ref();
        // SAFETY: The `StableAsRef` marker trait ensures that
        //         the impl of `AsRef<[u8]>` for `T` behaves sanely.
        unsafe { str::from_utf8_unchecked(b) }
    }
}

impl<T> ops::DerefMut for String<T>
    where T: AsRef<[u8]> + AsMut<[u8]>
{
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        let b = self.value.as_mut();
        // SAFETY: The `StableAsRef` marker trait ensures that
        //         the impl of `AsMut<[u8]>` for `T` behaves sanely.
        unsafe { str::from_utf8_unchecked_mut(b) }
    }
}

impl<T> borrow::Borrow<str> for String<T>
    where T: AsRef<[u8]>
{
    fn borrow(&self) -> &str {
        &*self
    }
}

impl From<::std::string::String> for String<::std::string::String> {
    fn from(value: ::std::string::String) -> Self {
        String { value }
    }
}

impl<T> Default for String<T>
    where T: Default + StableAsRef
{
    fn default() -> Self {
        String { value: T::default() }
    }
}

impl<T> TryFrom<T> for String<T>
    where T: AsRef<[u8]> + StableAsRef
{
    type Error = str::Utf8Error;

    fn try_from(value: T) -> Result<Self, Self::Error> {
        let _ = str::from_utf8(value.as_ref())?;
        Ok(String { value })
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for String<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(fmt)
    }
}

impl<T: AsRef<[u8]>> fmt::Display for String<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(fmt)
    }
}

/// Attempt to construct `Self` via a conversion.
///
/// This trait will be deprecated in favor of [std::convert::TryFrom] once it
/// reaches stable Rust.
pub trait TryFrom<T>: Sized + sealed::Sealed {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

impl<T> sealed::Sealed for String<T> {}

mod sealed {
    /// Private trait to this crate to prevent traits from being implemented in
    /// downstream crates.
    pub trait Sealed {}
}

/// Marker trait that indicates that a type is guaranteed safe to use as backing storage
/// for `String`.
///
/// In order to be safe, a storage type `T` needs to guarantee the following:
///
/// - If `T` implements `AsRef<[u8]>` and/or `AsMut<[u8]>`, the contents of `T` as visible
///   the byte slice returned by `as_ref` and `as_mut` may only be mutated through mutable
///   references or owned access. In other words, no use of interior mutability.
///
/// - If `T` implements `AsRef<[u8]>`, the `as_ref` method must always return the same
///   slice of bytes (unless the storage is mutated).
///
/// - If `T` implements `AsRef<[u8]>` and `AsMut<[u8]>`, the `as_mut` method must return
///   a mutable reference to the same slice of bytes as the `as_ref` method returns.
///
/// - If `T` implements `AsRef<[u8]>` and `Default`, the default value must represent the
///   empty byte sequence. In other words, `T::default().as_ref().len() == 0`.
///
/// - If `T` implements `AsRef<[u8]>` and `From<&[u8]>`, it must do so in such a way that
///   the byte slice returned by `as_ref` is equal to the byte slice provided to the `from`
///   method.
pub unsafe trait StableAsRef {}

unsafe impl<'a, T> StableAsRef for &'a T where T: StableAsRef {}
unsafe impl<'a, T> StableAsRef for &'a mut T where T: StableAsRef {}
unsafe impl<T> StableAsRef for Box<T> where T: StableAsRef {}
unsafe impl<T> StableAsRef for std::rc::Rc<T> where T: StableAsRef {}
unsafe impl<T> StableAsRef for std::sync::Arc<T> where T: StableAsRef {}

unsafe impl StableAsRef for std::string::String {}
unsafe impl StableAsRef for str {}
unsafe impl StableAsRef for Vec<u8> {}
unsafe impl StableAsRef for [u8] {}

#[cfg(feature = "bytes")]
unsafe impl StableAsRef for bytes::Bytes {}

#[cfg(feature = "bytes")]
unsafe impl StableAsRef for bytes::BytesMut {}

macro_rules! array_impls {
    ($($len:expr)+) => {
        $(
            unsafe impl StableAsRef for [u8; $len] {}
        )+
    }
}

array_impls!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_std_string() {
        let s: String<_> = "hello".to_string().into();
        assert_eq!(&s, "hello");
    }

    #[test]
    fn test_from_str() {
        let _: String<Vec<u8>> = String::from_str("nice str");
    }

    #[test]
    fn test_try_from_bytes() {
        let _ = String::try_from(b"nice bytes").unwrap();
    }
}
