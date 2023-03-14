//! Owned, growable wide strings.
//!
//! This module contains the [`UString`] strings and related types.

use crate::{UCStr, UCString, UChar, UStr, WideChar};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    string::String,
    vec::Vec,
};
use core::{
    borrow::{Borrow, BorrowMut},
    char, cmp,
    convert::Infallible,
    fmt::Write,
    iter::FromIterator,
    mem,
    ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut},
    slice::{self, SliceIndex},
    str::FromStr,
};

/// An owned, mutable "wide" string for FFI that is **not** nul-aware.
///
/// [`UString`] is not aware of nul values. Strings may or may not be nul-terminated, and may
/// contain invalid and ill-formed UTF-16 or UTF-32 data. These strings are intended to be used
/// with FFI functions that directly use string length, where the strings are known to have proper
/// nul-termination already, or where strings are merely being passed through without modification.
///
/// [`UCString`][crate::UCString] should be used instead if nul-aware strings are required.
///
/// [`UString`] can be converted to and from many other standard Rust string types, including
/// [`OsString`][std::ffi::OsString] and [`String`], making proper Unicode FFI safe and easy.
///
/// Please prefer using the type aliases [`U16String`], [`U32String`], or [`WideString`] to using
/// this type directly.
///
/// # Examples
///
/// The following example constructs a [`U16String`] and shows how to convert a [`U16String`] to a
/// regular Rust [`String`].
///
/// ```rust
/// use widestring::U16String;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U16String::from_str(s);
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
///
/// The same example using [`U32String`] instead:
///
/// ```rust
/// use widestring::U32String;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U32String::from_str(s);
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UString<C: UChar> {
    pub(crate) inner: Vec<C>,
}

impl<C: UChar> UString<C> {
    /// Constructs a new empty [`UString`].
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Constructs a [`UString`] from a vector.
    ///
    /// No checks are made on the contents of the vector. It may or may not be valid character data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let v = vec![84u16, 104u16, 101u16]; // 'T' 'h' 'e'
    /// # let cloned = v.clone();
    /// // Create a wide string from the vector
    /// let wstr = U16String::from_vec(v);
    /// # assert_eq!(wstr.into_vec(), cloned);
    /// ```
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let v = vec![84u32, 104u32, 101u32]; // 'T' 'h' 'e'
    /// # let cloned = v.clone();
    /// // Create a wide string from the vector
    /// let wstr = U32String::from_vec(v);
    /// # assert_eq!(wstr.into_vec(), cloned);
    /// ```
    #[inline]
    pub fn from_vec(raw: impl Into<Vec<C>>) -> Self {
        Self { inner: raw.into() }
    }

    /// Constructs a [`UString`] copy from a pointer and a length.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len`
    /// elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// # Panics
    ///
    /// Panics if `len` is greater than 0 but `p` is a null pointer.
    pub unsafe fn from_ptr(p: *const C, len: usize) -> Self {
        if len == 0 {
            return Self::new();
        }
        assert!(!p.is_null());
        let slice = slice::from_raw_parts(p, len);
        Self::from_vec(slice)
    }

    /// Constructs a [`UString`] with the given capacity.
    ///
    /// The string will be able to hold exactly `capacity` elements without reallocating.
    /// If `capacity` is set to 0, the string will not initially allocate.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Returns the capacity this [`UString`] can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Truncates the [`UString`] to zero length.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Reserves the capacity for at least `additional` more capacity to be inserted in the given
    /// [`UString`].
    ///
    /// More space may be reserved to avoid frequent allocations.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` more capacity to be inserted in the
    /// given [`UString`]. Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give more space than is requested. Therefore capacity can not
    /// be relied upon to be precisely minimal. Prefer [`reserve`][Self::reserve] if future
    /// insertions are expected.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Converts the string into a [`Vec`], consuming the string in the process.
    #[inline]
    pub fn into_vec(self) -> Vec<C> {
        self.inner
    }

    /// Converts to a [`UStr`] reference.
    #[inline]
    pub fn as_ustr(&self) -> &UStr<C> {
        UStr::from_slice(&self.inner)
    }

    /// Converts to a mutable [`UStr`] reference.
    #[inline]
    pub fn as_mut_ustr(&mut self) -> &mut UStr<C> {
        UStr::from_slice_mut(&mut self.inner)
    }

    /// Returns a [`Vec`] reference to the contents of this string.
    #[inline]
    pub fn as_vec(&self) -> &Vec<C> {
        &self.inner
    }

    /// Returns a mutable reference to the contents of this string.
    #[inline]
    pub fn as_mut_vec(&mut self) -> &mut Vec<C> {
        &mut self.inner
    }

    /// Extends the string with the given string slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// let mut wstr = U16String::from_str(s);
    /// let cloned = wstr.clone();
    /// // Push the clone to the end, repeating the string twice.
    /// wstr.push(cloned);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// let mut wstr = U32String::from_str(s);
    /// let cloned = wstr.clone();
    /// // Push the clone to the end, repeating the string twice.
    /// wstr.push(cloned);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[inline]
    pub fn push(&mut self, s: impl AsRef<UStr<C>>) {
        self.inner.extend_from_slice(&s.as_ref().inner)
    }

    /// Extends the string with the given slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// let mut wstr = U16String::from_str(s);
    /// let cloned = wstr.clone();
    /// // Push the clone to the end, repeating the string twice.
    /// wstr.push_slice(cloned);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// let mut wstr = U32String::from_str(s);
    /// let cloned = wstr.clone();
    /// // Push the clone to the end, repeating the string twice.
    /// wstr.push_slice(cloned);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[inline]
    pub fn push_slice(&mut self, s: impl AsRef<[C]>) {
        self.inner.extend_from_slice(s.as_ref())
    }

    /// Shrinks the capacity of the [`UString`] to match its length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    ///
    /// let mut s = U16String::from_str("foo");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    /// s.shrink_to_fit();
    /// assert_eq!(3, s.capacity());
    /// ```
    ///
    /// ```rust
    /// use widestring::U32String;
    ///
    /// let mut s = U32String::from_str("foo");
    ///
    /// s.reserve(100);
    /// assert!(s.capacity() >= 100);
    ///
    /// s.shrink_to_fit();
    /// assert_eq!(3, s.capacity());
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Converts this [`UString`] into a boxed [`UStr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use widestring::{U16String, U16Str};
    ///
    /// let s = U16String::from_str("hello");
    ///
    /// let b: Box<U16Str> = s.into_boxed_ustr();
    /// ```
    ///
    /// ```
    /// use widestring::{U32String, U32Str};
    ///
    /// let s = U32String::from_str("hello");
    ///
    /// let b: Box<U32Str> = s.into_boxed_ustr();
    /// ```
    pub fn into_boxed_ustr(self) -> Box<UStr<C>> {
        let rw = Box::into_raw(self.inner.into_boxed_slice()) as *mut UStr<C>;
        unsafe { Box::from_raw(rw) }
    }

    /// Shortens this string to the specified length.
    ///
    /// If `new_len` is greater than the string’s current length, this has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the string.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.inner.truncate(new_len)
    }

    /// Inserts a string slice into this string at a specified position.
    ///
    /// This is an _O(n)_ operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length.
    pub fn insert_ustr(&mut self, idx: usize, string: &UStr<C>) {
        assert!(idx <= self.len());
        self.inner
            .resize_with(self.len() + string.len(), Default::default);
        self.inner.copy_within(idx.., idx + string.len());
        self.inner[idx..].copy_from_slice(string.as_slice());
    }

    /// Splits the string into two at the given index.
    ///
    /// Returns a newly allocated string. `self` contains values `[0, at)`, and the returned string
    /// contains values `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at` is equal to or greater than the length of the string.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> UString<C> {
        Self::from_vec(self.inner.split_off(at))
    }
}

impl UString<u16> {
    /// Encodes a [`U16String`] copy from a [`str`].
    ///
    /// This makes a string copy of the [`str`]. Since [`str`] will always be valid UTF-8, the
    /// resulting [`U16String`] will also be valid UTF-16.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U16String::from_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), s);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str<S: AsRef<str> + ?Sized>(s: &S) -> Self {
        Self {
            inner: s.as_ref().encode_utf16().collect(),
        }
    }

    /// Encodes a [`U16String`] copy from an [`OsStr`][std::ffi::OsStr].
    ///
    /// This makes a string copy of the [`OsStr`][std::ffi::OsStr]. Since [`OsStr`][std::ffi::OsStr]
    /// makes no  guarantees that it is valid data, there is no guarantee that the resulting
    /// [`U16String`] will be valid UTF-16.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms (such as
    /// windows) no changes to the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U16String::from_os_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), s);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub fn from_os_str<S: AsRef<std::ffi::OsStr> + ?Sized>(s: &S) -> Self {
        Self {
            inner: crate::platform::os_to_wide(s.as_ref()),
        }
    }

    /// Extends the string with the given string slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// let mut wstr = U16String::from_str(s);
    /// // Push the original to the end, repeating the string twice.
    /// wstr.push_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.inner.extend(s.as_ref().encode_utf16())
    }

    /// Extends the string with the given string slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16String;
    /// let s = "MyString";
    /// let mut wstr = U16String::from_str(s);
    /// // Push the original to the end, repeating the string twice.
    /// wstr.push_os_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub fn push_os_str(&mut self, s: impl AsRef<std::ffi::OsStr>) {
        self.inner.extend(crate::platform::os_to_wide(s.as_ref()))
    }

    /// Appends the given [`char`][prim@char] to the end of this string.
    #[inline]
    pub fn push_char(&mut self, c: char) {
        let mut buf = [0; 2];
        self.inner.extend_from_slice(c.encode_utf16(&mut buf))
    }

    /// Removes the last character or unpaired surrogate from the string buffer and returns it.
    ///
    /// Returns `None` if this String is empty. Otherwise, returns the character cast to a
    /// [`u32`][prim@u32] or the value of the unpaired surrogate as a [`u32`][prim@u32] value.
    pub fn pop(&mut self) -> Option<u32> {
        match self.inner.pop() {
            Some(low) if crate::is_utf16_surrogate(low) => {
                if !crate::is_utf16_low_surrogate(low) || self.inner.is_empty() {
                    Some(low as u32)
                } else {
                    let high = self.inner[self.len()];
                    if crate::is_utf16_high_surrogate(high) {
                        self.inner.pop();
                        let buf = [high, low];
                        Some(
                            char::decode_utf16(buf.iter().copied())
                                .next()
                                .unwrap()
                                .unwrap() as u32,
                        )
                    } else {
                        Some(low as u32)
                    }
                }
            }
            Some(u) => Some(u as u32),
            None => None,
        }
    }

    /// Removes a [`char`][prim@char] or unpaired surrogate from this string at a position and
    /// returns it as a [`u32`][prim@u32].
    ///
    /// This is an _O(n)_ operation, as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the string's length.
    pub fn remove(&mut self, idx: usize) -> u32 {
        let slice = &self.inner[idx..];
        let c = char::decode_utf16(slice.iter().copied()).next().unwrap();
        let clen = c.as_ref().map(|c| c.len_utf16()).unwrap_or(1);
        let c = c
            .map(|c| c as u32)
            .unwrap_or_else(|_| self.inner[idx] as u32);
        self.inner.drain(idx..idx + clen);
        c
    }

    /// Inserts a character into this string at a specified position.
    ///
    /// This is an _O(n)_ operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length.
    pub fn insert(&mut self, idx: usize, c: char) {
        assert!(idx <= self.len());
        let mut buf = [0; 2];
        let slice = c.encode_utf16(&mut buf);
        self.inner.resize(self.len() + slice.len(), 0);
        self.inner.copy_within(idx.., idx + slice.len());
        self.inner[idx..].copy_from_slice(slice);
    }
}

impl UString<u32> {
    /// Constructs a [`U32String`] from a [`char`][prim@char] vector.
    ///
    /// No checks are made on the contents of the vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let v: Vec<char> = "Test".chars().collect();
    /// # let cloned: Vec<u32> = v.iter().map(|&c| c as u32).collect();
    /// // Create a wide string from the vector
    /// let wstr = U32String::from_chars(v);
    /// # assert_eq!(wstr.into_vec(), cloned);
    /// ```
    pub fn from_chars(raw: impl Into<Vec<char>>) -> Self {
        let mut chars = raw.into();
        Self {
            inner: unsafe {
                let ptr = chars.as_mut_ptr() as *mut u32;
                let len = chars.len();
                let cap = chars.capacity();
                mem::forget(chars);
                Vec::from_raw_parts(ptr, len, cap)
            },
        }
    }

    /// Encodes a [`U32String`] copy from a [`str`].
    ///
    /// This makes a string copy of the [`str`]. Since [`str`] will always be valid UTF-8, the
    /// resulting [`U32String`] will also be valid UTF-32.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U32String::from_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), s);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str<S: AsRef<str> + ?Sized>(s: &S) -> Self {
        let v: Vec<char> = s.as_ref().chars().collect();
        Self::from_chars(v)
    }

    /// Encodes a [`U32String`] copy from an [`OsStr`][std::ffi::OsStr].
    ///
    /// This makes a string copy of the [`OsStr`][std::ffi::OsStr]. Since [`OsStr`][std::ffi::OsStr]
    /// makes no guarantees that it is valid data, there is no guarantee that the resulting
    /// [`U32String`] will be valid UTF-32.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms no changes to
    /// the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U32String::from_os_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), s);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn from_os_str<S: AsRef<std::ffi::OsStr> + ?Sized>(s: &S) -> Self {
        let v: Vec<char> = s.as_ref().to_string_lossy().chars().collect();
        Self::from_chars(v)
    }

    /// Constructs a [`U32String`] from a [`char`][prim@char] pointer and a length.
    ///
    /// The `len` argument is the number of `char` elements, **not** the number of bytes.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len`
    /// elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// # Panics
    ///
    /// Panics if `len` is greater than 0 but `p` is a null pointer.
    #[inline]
    pub unsafe fn from_char_ptr(p: *const char, len: usize) -> Self {
        Self::from_ptr(p as *const u32, len)
    }

    /// Extends the string with the given string slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// let mut wstr = U32String::from_str(s);
    /// // Push the original to the end, repeating the string twice.
    /// wstr.push_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.inner.extend(s.as_ref().chars().map(|c| c as u32))
    }

    /// Extends the string with the given string slice.
    ///
    /// No checks are performed on the strings. It is possible to end up nul values inside the
    /// string, and it is up to the caller to determine if that is acceptable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32String;
    /// let s = "MyString";
    /// let mut wstr = U32String::from_str(s);
    /// // Push the original to the end, repeating the string twice.
    /// wstr.push_os_str(s);
    ///
    /// assert_eq!(wstr.to_string().unwrap(), "MyStringMyString");
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub fn push_os_str(&mut self, s: impl AsRef<std::ffi::OsStr>) {
        self.inner
            .extend(s.as_ref().to_string_lossy().chars().map(|c| c as u32))
    }

    /// Appends the given [`char`][prim@char] to the end of this string.
    #[inline]
    pub fn push_char(&mut self, c: char) {
        self.inner.push(c as u32);
    }

    /// Removes the last value from the string buffer and returns it.
    ///
    /// Returns `None` if this String is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<u32> {
        self.inner.pop()
    }

    /// Removes a value from this string at a position and returns it.
    ///
    /// This is an _O(n)_ operation, as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the string's length.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> u32 {
        self.inner.remove(idx)
    }

    /// Inserts a character into this string at a specified position.
    ///
    /// This is an _O(n)_ operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length.
    #[inline]
    pub fn insert(&mut self, idx: usize, c: char) {
        self.inner.insert(idx, c as u32)
    }
}

impl<C: UChar> Add<&UStr<C>> for UString<C> {
    type Output = UString<C>;

    #[inline]
    fn add(mut self, rhs: &UStr<C>) -> Self::Output {
        self.push(rhs);
        self
    }
}

impl<C: UChar> Add<&UCStr<C>> for UString<C> {
    type Output = UString<C>;

    #[inline]
    fn add(mut self, rhs: &UCStr<C>) -> Self::Output {
        self.push(rhs);
        self
    }
}

impl Add<&str> for U16String {
    type Output = U16String;

    #[inline]
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl Add<&str> for U32String {
    type Output = U32String;

    #[inline]
    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl<C: UChar> AddAssign<&UStr<C>> for UString<C> {
    #[inline]
    fn add_assign(&mut self, rhs: &UStr<C>) {
        self.push(rhs)
    }
}

impl<C: UChar> AddAssign<&UCStr<C>> for UString<C> {
    #[inline]
    fn add_assign(&mut self, rhs: &UCStr<C>) {
        self.push(rhs)
    }
}

impl AddAssign<&str> for U16String {
    #[inline]
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl AddAssign<&str> for U32String {
    #[inline]
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl<C: UChar> AsMut<UStr<C>> for UString<C> {
    #[inline]
    fn as_mut(&mut self) -> &mut UStr<C> {
        self.as_mut_ustr()
    }
}

impl<C: UChar> AsMut<[C]> for UString<C> {
    #[inline]
    fn as_mut(&mut self) -> &mut [C] {
        self.as_mut_slice()
    }
}

impl<C: UChar> AsRef<UStr<C>> for UString<C> {
    #[inline]
    fn as_ref(&self) -> &UStr<C> {
        self.as_ustr()
    }
}

impl<C: UChar> AsRef<[C]> for UString<C> {
    #[inline]
    fn as_ref(&self) -> &[C] {
        self.as_slice()
    }
}

impl<C: UChar> Borrow<UStr<C>> for UString<C> {
    #[inline]
    fn borrow(&self) -> &UStr<C> {
        self.as_ustr()
    }
}

impl<C: UChar> BorrowMut<UStr<C>> for UString<C> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut UStr<C> {
        self.as_mut_ustr()
    }
}

impl core::fmt::Debug for U16String {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u16(self.as_slice(), f)
    }
}

impl core::fmt::Debug for U32String {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u32(self.as_slice(), f)
    }
}

impl<C: UChar> Default for Box<UStr<C>> {
    #[inline]
    fn default() -> Self {
        let boxed: Box<[C]> = Box::from([]);
        let rw = Box::into_raw(boxed) as *mut UStr<C>;
        unsafe { Box::from_raw(rw) }
    }
}

impl<C: UChar> Deref for UString<C> {
    type Target = UStr<C>;

    #[inline]
    fn deref(&self) -> &UStr<C> {
        self.as_ustr()
    }
}

impl<C: UChar> DerefMut for UString<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_ustr()
    }
}

impl<'a, C: UChar> Extend<&'a UStr<C>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a UStr<C>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s))
    }
}

impl<'a, C: UChar> Extend<&'a UCStr<C>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a UCStr<C>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s))
    }
}

impl<'a> Extend<&'a str> for U16String {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s))
    }
}

impl<'a> Extend<&'a str> for U32String {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s))
    }
}

impl<C: UChar> Extend<UString<C>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = UString<C>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s))
    }
}

impl<C: UChar> Extend<UCString<C>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = UCString<C>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s.as_ucstr()))
    }
}

impl Extend<String> for U16String {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s))
    }
}

impl Extend<String> for U32String {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s))
    }
}

impl Extend<char> for U16String {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.reserve(lower_bound);
        iter.for_each(|c| self.push_char(c));
    }
}

impl Extend<char> for U32String {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.reserve(lower_bound);
        iter.for_each(|c| self.push_char(c));
    }
}

impl<'a> Extend<&'a char> for U16String {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied())
    }
}

impl<'a> Extend<&'a char> for U32String {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied())
    }
}

impl<C: UChar> Extend<Box<UStr<C>>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<UStr<C>>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s))
    }
}

impl<'a, C: UChar> Extend<Cow<'a, UStr<C>>> for UString<C> {
    #[inline]
    fn extend<T: IntoIterator<Item = Cow<'a, UStr<C>>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push(s))
    }
}

impl<C: UChar> From<UString<C>> for Vec<C> {
    #[inline]
    fn from(value: UString<C>) -> Self {
        value.into_vec()
    }
}

impl<'a> From<UString<u16>> for Cow<'a, UStr<u16>> {
    #[inline]
    fn from(s: UString<u16>) -> Self {
        Cow::Owned(s)
    }
}

impl<'a> From<UString<u32>> for Cow<'a, UStr<u32>> {
    #[inline]
    fn from(s: UString<u32>) -> Self {
        Cow::Owned(s)
    }
}

impl From<Vec<u16>> for UString<u16> {
    #[inline]
    fn from(value: Vec<u16>) -> Self {
        Self::from_vec(value)
    }
}

impl From<Vec<u32>> for UString<u32> {
    #[inline]
    fn from(value: Vec<u32>) -> Self {
        Self::from_vec(value)
    }
}

impl From<Vec<char>> for UString<u32> {
    #[inline]
    fn from(value: Vec<char>) -> Self {
        Self::from_chars(value)
    }
}

impl From<String> for UString<u16> {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl From<String> for UString<u32> {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl From<&str> for UString<u16> {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<&str> for UString<u32> {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(feature = "std")]
impl From<std::ffi::OsString> for UString<u16> {
    #[inline]
    fn from(s: std::ffi::OsString) -> Self {
        Self::from_os_str(&s)
    }
}

#[cfg(feature = "std")]
impl From<std::ffi::OsString> for UString<u32> {
    #[inline]
    fn from(s: std::ffi::OsString) -> Self {
        Self::from_os_str(&s)
    }
}

#[cfg(feature = "std")]
impl From<UString<u16>> for std::ffi::OsString {
    #[inline]
    fn from(s: UString<u16>) -> Self {
        s.to_os_string()
    }
}

#[cfg(feature = "std")]
impl From<UString<u32>> for std::ffi::OsString {
    #[inline]
    fn from(s: UString<u32>) -> Self {
        s.to_os_string()
    }
}

impl<'a, C: UChar, T: ?Sized + AsRef<UStr<C>>> From<&'a T> for UString<C> {
    #[inline]
    fn from(s: &'a T) -> Self {
        s.as_ref().to_ustring()
    }
}

impl<'a> From<&'a UStr<u16>> for Cow<'a, UStr<u16>> {
    #[inline]
    fn from(s: &'a UStr<u16>) -> Self {
        Cow::Borrowed(s)
    }
}

impl<'a> From<&'a UStr<u32>> for Cow<'a, UStr<u32>> {
    #[inline]
    fn from(s: &'a UStr<u32>) -> Self {
        Cow::Borrowed(s)
    }
}

impl<'a, C: UChar> From<&'a UStr<C>> for Box<UStr<C>> {
    fn from(s: &'a UStr<C>) -> Self {
        let boxed: Box<[C]> = Box::from(&s.inner);
        let rw = Box::into_raw(boxed) as *mut UStr<C>;
        unsafe { Box::from_raw(rw) }
    }
}

impl<C: UChar> From<Box<UStr<C>>> for UString<C> {
    #[inline]
    fn from(boxed: Box<UStr<C>>) -> Self {
        boxed.into_ustring()
    }
}

impl<C: UChar> From<UString<C>> for Box<UStr<C>> {
    #[inline]
    fn from(s: UString<C>) -> Self {
        s.into_boxed_ustr()
    }
}

impl<'a, C: UChar + 'a> FromIterator<&'a UStr<C>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a UStr<C>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a, C: UChar + 'a> FromIterator<&'a UCStr<C>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a UCStr<C>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a> FromIterator<&'a str> for U16String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a> FromIterator<&'a str> for U32String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<C: UChar> FromIterator<UString<C>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UString<C>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<C: UChar> FromIterator<UCString<C>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UCString<C>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl FromIterator<String> for U16String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl FromIterator<String> for U32String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl FromIterator<char> for U16String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl FromIterator<char> for U32String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a> FromIterator<&'a char> for U16String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a> FromIterator<&'a char> for U32String {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<C: UChar> FromIterator<Box<UStr<C>>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Box<UStr<C>>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl<'a, C: UChar + 'a> FromIterator<Cow<'a, UStr<C>>> for UString<C> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Cow<'a, UStr<C>>>>(iter: T) -> Self {
        let mut string = Self::new();
        string.extend(iter);
        string
    }
}

impl FromStr for U16String {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

impl FromStr for U32String {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

impl<C: UChar, I> Index<I> for UString<C>
where
    I: SliceIndex<[C], Output = [C]>,
{
    type Output = UStr<C>;

    #[inline]
    fn index(&self, index: I) -> &UStr<C> {
        &self.as_ustr()[index]
    }
}

impl<C: UChar, I> IndexMut<I> for UString<C>
where
    I: SliceIndex<[C], Output = [C]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_ustr()[index]
    }
}

impl<C: UChar> PartialEq<UStr<C>> for UString<C> {
    #[inline]
    fn eq(&self, other: &UStr<C>) -> bool {
        self.as_ustr() == other
    }
}

impl<C: UChar> PartialEq<UCStr<C>> for UString<C> {
    #[inline]
    fn eq(&self, other: &UCStr<C>) -> bool {
        self.as_ustr() == other
    }
}

impl<C: UChar> PartialEq<UCString<C>> for UString<C> {
    #[inline]
    fn eq(&self, other: &UCString<C>) -> bool {
        self.as_ustr() == other.as_ucstr()
    }
}

impl<'a, C: UChar> PartialEq<&'a UStr<C>> for UString<C> {
    #[inline]
    fn eq(&self, other: &&'a UStr<C>) -> bool {
        self.as_ustr() == *other
    }
}

impl<'a, C: UChar> PartialEq<&'a UCStr<C>> for UString<C> {
    #[inline]
    fn eq(&self, other: &&'a UCStr<C>) -> bool {
        self.as_ustr() == *other
    }
}

impl<'a, C: UChar> PartialEq<Cow<'a, UStr<C>>> for UString<C> {
    #[inline]
    fn eq(&self, other: &Cow<'a, UStr<C>>) -> bool {
        self.as_ustr() == other.as_ref()
    }
}

impl<'a, C: UChar> PartialEq<Cow<'a, UCStr<C>>> for UString<C> {
    #[inline]
    fn eq(&self, other: &Cow<'a, UCStr<C>>) -> bool {
        self.as_ustr() == other.as_ref()
    }
}

impl<C: UChar> PartialOrd<UStr<C>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UStr<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other)
    }
}

impl<C: UChar> PartialOrd<UCStr<C>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UCStr<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other)
    }
}

impl<'a, C: UChar> PartialOrd<&'a UStr<C>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &&'a UStr<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(*other)
    }
}

impl<'a, C: UChar> PartialOrd<&'a UCStr<C>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &&'a UCStr<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(*other)
    }
}

impl<'a, C: UChar> PartialOrd<Cow<'a, UStr<C>>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &Cow<'a, UStr<C>>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other.as_ref())
    }
}

impl<'a, C: UChar> PartialOrd<Cow<'a, UCStr<C>>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &Cow<'a, UCStr<C>>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other.as_ref())
    }
}

impl<C: UChar> PartialOrd<UCString<C>> for UString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UCString<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other.as_ucstr())
    }
}

impl<C: UChar> ToOwned for UStr<C> {
    type Owned = UString<C>;

    #[inline]
    fn to_owned(&self) -> UString<C> {
        self.to_ustring()
    }
}

impl Write for U16String {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.push_char(c);
        Ok(())
    }
}

/// An owned, mutable "wide" string for FFI that is **not** nul-aware.
///
/// [`U16String`] is not aware of nul values. Strings may or may not be nul-terminated, and may
/// contain invalid and ill-formed UTF-16 data. These strings are intended to be used with
/// FFI functions that directly use string length, where the strings are known to have proper
/// nul-termination already, or where strings are merely being passed through without modification.
///
/// [`U16CString`][crate::U16CString] should be used instead if nul-aware strings are required.
///
/// [`U16String`] can be converted to and from many other standard Rust string types, including
/// [`OsString`][std::ffi::OsString] and [`String`], making proper Unicode FFI safe and easy.
///
/// # Examples
///
/// The following example constructs a [`U16String`] and shows how to convert a [`U16String`] to a
/// regular Rust [`String`].
///
/// ```rust
/// use widestring::U16String;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U16String::from_str(s);
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
pub type U16String = UString<u16>;

/// An owned, mutable 32-bit wide string for FFI that is **not** nul-aware.
///
/// [`U32String`] is not aware of nul values. Strings may or may not be nul-terminated, and may
/// contain invalid and ill-formed UTF-32 data. These strings are intended to be used with
/// FFI functions that directly use string length, where the strings are known to have proper
/// nul-termination already, or where strings are merely being passed through without modification.
///
/// [`U32CString`][crate::U32CString] should be used instead if nul-aware 32-bit strings are
/// required.
///
/// [`U32String`] can be converted to and from many other standard Rust string types, including
/// [`OsString`][std::ffi::OsString] and [`String`], making proper Unicode FFI safe and easy.
///
/// # Examples
///
/// The following example constructs a [`U32String`] and shows how to convert a [`U32String`] to a
/// regular Rust [`String`].
///
/// ```rust
/// use widestring::U32String;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U32String::from_str(s);
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
pub type U32String = UString<u32>;

/// Alias for [`U16String`] or [`U32String`] depending on platform. Intended to match typical C
/// `wchar_t` size on platform.
pub type WideString = UString<WideChar>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(clippy::write_literal)]
    fn number_to_string() {
        let mut s = U16String::new();
        write!(s, "{}", 1234).unwrap();
        assert_eq!(s, U16String::from_str("1234"));
    }
}
