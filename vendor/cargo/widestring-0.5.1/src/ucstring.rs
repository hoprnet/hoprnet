//! C-style owned, growable wide strings.
//!
//! This module contains the [`UCString`] strings and related types.

use crate::{error::ContainsNul, UCStr, UChar, UStr, UString, WideChar};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    vec::Vec,
};
use core::{
    borrow::{Borrow, BorrowMut},
    cmp, mem,
    ops::{Deref, DerefMut, Index, IndexMut, RangeFull},
    ptr, slice,
};

/// An owned, mutable C-style "wide" string for FFI that is nul-aware and nul-terminated.
///
/// [`UCString`] is aware of nul values. Unless unchecked conversions are used, all [`UCString`]
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-16 or UTF-32 data. These strings are
/// intended to be used with FFI functions such as Windows API that may require nul-terminated
/// strings.
///
/// [`UCString`] can be converted to and from many other string types, including [`UString`],
/// [`OsString`][std::ffi::OsString], and [`String`], making proper Unicode FFI safe and easy.
///
/// Please prefer using the type aliases [`U16CString`], [`U32CString`], or [`WideCString`] to using
/// this type directly.
///
/// # Examples
///
/// The following example constructs a [`U16CString`] and shows how to convert a [`U16CString`] to a
/// regular Rust [`String`].
///
/// ```rust
/// use widestring::U16CString;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U16CString::from_str(s).unwrap();
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
///
/// The same example using [`U32CString`]:
///
/// ```rust
/// use widestring::U32CString;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U32CString::from_str(s).unwrap();
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UCString<C: UChar> {
    pub(crate) inner: Box<[C]>,
}

impl<C: UChar> UCString<C> {
    /// Constructs a [`UCString`] from a container of wide character data.
    ///
    /// This method will consume the provided data and use the underlying elements to construct a
    /// new string. The data will be scanned for invalid interior nul values.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value that is not the
    /// terminating nul.
    /// The returned error will contain the original [`Vec`] as well as the position of the nul
    /// value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let v = vec![84u16, 104u16, 101u16]; // 'T' 'h' 'e'
    /// # let cloned = v.clone();
    /// // Create a wide string from the vector
    /// let wcstr = U16CString::from_vec(v).unwrap();
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v = vec![84u32, 104u32, 101u32]; // 'T' 'h' 'e'
    /// # let cloned = v.clone();
    /// // Create a wide string from the vector
    /// let wcstr = U32CString::from_vec(v).unwrap();
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    ///
    /// Empty vectors are valid and will return an empty string with a nul terminator:
    ///
    /// ```
    /// use widestring::U16CString;
    /// let wcstr = U16CString::from_vec(vec![]).unwrap();
    /// assert_eq!(wcstr, U16CString::default());
    /// ```
    ///
    /// ```
    /// use widestring::U32CString;
    /// let wcstr = U32CString::from_vec(vec![]).unwrap();
    /// assert_eq!(wcstr, U32CString::default());
    /// ```
    ///
    /// The following example demonstrates errors from nul values in a vector.
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let v = vec![84u16, 0u16, 104u16, 101u16]; // 'T' NUL 'h' 'e'
    /// // Create a wide string from the vector
    /// let res = U16CString::from_vec(v);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 1);
    /// ```
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v = vec![84u32, 0u32, 104u32, 101u32]; // 'T' NUL 'h' 'e'
    /// // Create a wide string from the vector
    /// let res = U32CString::from_vec(v);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 1);
    /// ```
    pub fn from_vec(v: impl Into<Vec<C>>) -> Result<Self, ContainsNul<C>> {
        let v = v.into();
        // Check for nul vals, ignoring nul terminator
        match v.iter().position(|&val| val == UChar::NUL) {
            None => Ok(unsafe { Self::from_vec_unchecked(v) }),
            Some(pos) if pos == v.len() - 1 => Ok(unsafe { Self::from_vec_unchecked(v) }),
            Some(pos) => Err(ContainsNul::new(pos, v)),
        }
    }

    /// Constructs a [`UCString`] from a container of wide character data, truncating at the first
    /// nul terminator.
    ///
    /// The string will be truncated at the first nul value in the data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let v = vec![84u16, 104u16, 101u16, 0u16]; // 'T' 'h' 'e' NUL
    /// # let cloned = v[..3].to_owned();
    /// // Create a wide string from the vector
    /// let wcstr = U16CString::from_vec_truncate(v);
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v = vec![84u32, 104u32, 101u32, 0u32]; // 'T' 'h' 'e' NUL
    /// # let cloned = v[..3].to_owned();
    /// // Create a wide string from the vector
    /// let wcstr = U32CString::from_vec_truncate(v);
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    pub fn from_vec_truncate(v: impl Into<Vec<C>>) -> Self {
        let mut v = v.into();
        // Check for nul vals
        if let Some(pos) = v.iter().position(|&val| val == UChar::NUL) {
            v.truncate(pos + 1);
        }
        unsafe { Self::from_vec_unchecked(v) }
    }

    /// Constructs a [`UCString`] from a vector without checking for interior nul values.
    ///
    /// A terminating nul value will be appended if the vector does not already have a terminating
    /// nul.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_vec`][Self::from_vec] except that no runtime assertion
    /// is made that `v` contains no interior nul values. Providing a vector with any nul values
    /// that are not the last value in the vector will result in an invalid [`UCString`].
    pub unsafe fn from_vec_unchecked(v: impl Into<Vec<C>>) -> Self {
        let mut v = v.into();
        match v.last() {
            None => v.push(UChar::NUL),
            Some(&c) if c != UChar::NUL => v.push(UChar::NUL),
            Some(_) => (),
        }
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Constructs a [`UCString`] from anything that can be converted to a [`UStr`].
    ///
    /// The string will be scanned for invalid interior nul values.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value that is not the
    /// terminating nul.
    /// The returned error will contain a [`Vec`] as well as the position of the nul value.
    #[inline]
    pub fn from_ustr(s: impl AsRef<UStr<C>>) -> Result<Self, ContainsNul<C>> {
        Self::from_vec(s.as_ref().as_slice())
    }

    /// Constructs a [`UCString`] from anything that can be converted to a [`UStr`], truncating at
    /// the first nul terminator.
    ///
    /// The string will be truncated at the first nul value in the string.
    #[inline]
    pub fn from_ustr_truncate(s: impl AsRef<UStr<C>>) -> Self {
        Self::from_vec_truncate(s.as_ref().as_slice())
    }

    /// Constructs a [`UCString`] from anything that can be converted to a [`UStr`], without
    /// scanning for invalid nul values.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_ustr`][Self::from_ustr] except that no runtime assertion
    /// is made that `v` contains no interior nul values. Providing a string with any nul values
    /// that are not the last value in the vector will result in an invalid [`UCString`].
    #[inline]
    pub unsafe fn from_ustr_unchecked(s: impl AsRef<UStr<C>>) -> Self {
        Self::from_vec_unchecked(s.as_ref().as_slice())
    }

    /// Constructs a new [`UCString`] copied from a nul-terminated string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// If you wish to avoid copying the string pointer, use [`UCStr::from_ptr_str`] instead.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// # Panics
    ///
    /// This function panics if `p` is null.
    ///
    /// # Caveat
    ///
    /// The lifetime for the returned string is inferred from its usage. To prevent accidental
    /// misuse, it's suggested to tie the lifetime to whichever source lifetime is safe in the
    /// context, such as by providing a helper function taking the lifetime of a host value for the
    /// string, or by explicit annotation.
    #[inline]
    pub unsafe fn from_ptr_str(p: *const C) -> Self {
        UCStr::from_ptr_str(p).to_ucstring()
    }

    /// Constructs a [`UCString`] copied from a pointer and a length, checking for invalid interior
    /// nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. If `len` is `0`, `p` is allowed to be a
    /// null pointer.
    ///
    /// The resulting string will always be nul-terminated even if the pointer data is not.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found. To
    /// avoid scanning for interior nuls, [`from_ptr_unchecked`][Self::from_ptr_unchecked] may be
    /// used instead.
    /// The returned error will contain a [`Vec`] as well as the position of the nul value.
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
    pub unsafe fn from_ptr(p: *const C, len: usize) -> Result<Self, ContainsNul<C>> {
        if len == 0 {
            return Ok(Self::default());
        }
        assert!(!p.is_null());
        let slice = slice::from_raw_parts(p, len);
        Self::from_vec(slice)
    }

    /// Constructs a [`UCString`] copied from a pointer and a length, truncating at the first nul
    /// terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`. If no
    /// nul value is found, the whole string of length `len` is used, and a new nul-terminator
    /// will be added to the resulting string. If `len` is `0`, `p` is allowed to be a null pointer.
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
    pub unsafe fn from_ptr_truncate(p: *const C, len: usize) -> Self {
        if len == 0 {
            return Self::default();
        }
        assert!(!p.is_null());
        let slice = slice::from_raw_parts(p, len);
        Self::from_vec_truncate(slice)
    }

    /// Constructs a [`UCString`] copied from a pointer and a length without checking for any nul
    /// values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. If `len` is `0`, `p` is allowed to be a
    /// null pointer.
    ///
    /// The resulting string will always be nul-terminated even if the pointer data is not.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len`
    /// elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// will result in an invalid [`UCString`].
    ///
    /// # Panics
    ///
    /// Panics if `len` is greater than 0 but `p` is a null pointer.
    pub unsafe fn from_ptr_unchecked(p: *const C, len: usize) -> Self {
        if len == 0 {
            return Self::default();
        }
        assert!(!p.is_null());
        let slice = slice::from_raw_parts(p, len);
        Self::from_vec_unchecked(slice)
    }

    /// Converts to a [`UCStr`] reference.
    #[inline]
    pub fn as_ucstr(&self) -> &UCStr<C> {
        UCStr::from_inner(&self.inner)
    }

    /// Converts to a mutable [`UCStr`] reference.
    #[inline]
    pub fn as_mut_ucstr(&mut self) -> &mut UCStr<C> {
        UCStr::from_inner_mut(&mut self.inner)
    }

    /// Converts this string into a [`UString`] without a nul terminator.
    ///
    /// The resulting string will **not** contain a nul-terminator, and will contain no other nul
    /// values.
    #[inline]
    pub fn into_ustring(self) -> UString<C> {
        UString::from_vec(self.into_vec())
    }

    /// Converts this string into a [`UString`] with a nul terminator.
    ///
    /// The resulting vector will contain a nul-terminator and no interior nul values.
    #[inline]
    pub fn into_ustring_with_nul(self) -> UString<C> {
        UString::from_vec(self.into_vec_with_nul())
    }

    /// Converts the string into a [`Vec`] without a nul terminator, consuming the string in
    /// the process.
    ///
    /// The resulting vector will **not** contain a nul-terminator, and will contain no other nul
    /// values.
    #[inline]
    pub fn into_vec(self) -> Vec<C> {
        let mut v = self.into_inner().into_vec();
        v.pop();
        v
    }

    /// Converts the string into a [`Vec`], consuming the string in the process.
    ///
    /// The resulting vector will contain a nul-terminator and no interior nul values.
    #[inline]
    pub fn into_vec_with_nul(self) -> Vec<C> {
        self.into_inner().into_vec()
    }

    /// Transfers ownership of the string to a C caller.
    ///
    /// # Safety
    ///
    /// The pointer _must_ be returned to Rust and reconstituted using
    /// [`from_raw`][Self::from_raw] to be properly deallocated. Specifically, one should _not_ use
    /// the standard C `free` function to deallocate this string. Failure to call
    /// [`from_raw`][Self::from_raw] will lead to a memory leak.
    #[inline]
    pub fn into_raw(self) -> *mut C {
        Box::into_raw(self.into_inner()) as *mut C
    }

    /// Retakes ownership of a [`UCString`] that was transferred to C.
    ///
    /// This should only be used in combination with [`into_raw`][Self::into_raw]. To construct a
    /// new [`UCString`] from a pointer, use [`from_ptr_str`][Self::from_ptr_str].
    ///
    /// # Safety
    ///
    /// This should only ever be called with a pointer that was earlier obtained by calling
    /// [`into_raw`][Self::into_raw]. Additionally, the length of the string will be recalculated
    /// from the pointer by scanning for the nul-terminator.
    ///
    /// # Panics
    ///
    /// Panics if `p` is a null pointer.
    pub unsafe fn from_raw(p: *mut C) -> Self {
        assert!(!p.is_null());
        let mut i: isize = 0;
        while *p.offset(i) != UChar::NUL {
            i += 1;
        }
        let slice = slice::from_raw_parts_mut(p, i as usize + 1);
        Self {
            inner: Box::from_raw(slice),
        }
    }

    /// Converts this [`UCString`] into a boxed [`UCStr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use widestring::{U16CString, U16CStr};
    ///
    /// let mut v = vec![102u16, 111u16, 111u16]; // "foo"
    /// let c_string = U16CString::from_vec(v.clone()).unwrap();
    /// let boxed = c_string.into_boxed_ucstr();
    /// v.push(0);
    /// assert_eq!(&*boxed, U16CStr::from_slice(&v).unwrap());
    /// ```
    ///
    /// ```
    /// use widestring::{U32CString, U32CStr};
    ///
    /// let mut v = vec![102u32, 111u32, 111u32]; // "foo"
    /// let c_string = U32CString::from_vec(v.clone()).unwrap();
    /// let boxed = c_string.into_boxed_ucstr();
    /// v.push(0);
    /// assert_eq!(&*boxed, U32CStr::from_slice(&v).unwrap());
    /// ```
    #[inline]
    pub fn into_boxed_ucstr(self) -> Box<UCStr<C>> {
        unsafe { Box::from_raw(Box::into_raw(self.into_inner()) as *mut UCStr<C>) }
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_vec` instead"]
    pub fn new(v: impl Into<Vec<C>>) -> Result<Self, ContainsNul<C>> {
        Self::from_vec(v)
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_vec_truncate` instead"]
    pub fn from_vec_with_nul(v: impl Into<Vec<C>>) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_vec_truncate(v))
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_ustr_truncate` instead"]
    pub fn from_ustr_with_nul(s: impl AsRef<UStr<C>>) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_ustr_truncate(s))
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_ptr_truncate` instead"]
    pub unsafe fn from_ptr_with_nul(
        p: *const C,
        len: usize,
    ) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_ptr_truncate(p, len))
    }

    #[doc(hidden)]
    #[deprecated = "use `from_vec_unchecked` instead"]
    pub unsafe fn from_vec_with_nul_unchecked(v: impl Into<Vec<C>>) -> Self {
        Self::from_vec_unchecked(v)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_ustr_unchecked` instead"]
    pub unsafe fn from_ustr_with_nul_unchecked(v: impl AsRef<UStr<C>>) -> Self {
        Self::from_ustr_unchecked(v)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_ptr_unchecked` instead"]
    pub unsafe fn from_ptr_with_nul_unchecked(p: *const C, len: usize) -> Self {
        Self::from_ptr_unchecked(p, len)
    }

    /// Bypass "move out of struct which implements [`Drop`] trait" restriction.
    fn into_inner(self) -> Box<[C]> {
        let result = unsafe { ptr::read(&self.inner) };
        mem::forget(self);
        result
    }
}

impl UCString<u16> {
    /// Encodes a [`U16CString`] copied from a [`str`].
    ///
    /// The string will be scanned for nul values, which are invalid anywhere except the final
    /// character.
    ///
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value anywhere except the
    /// final position.
    /// The returned error will contain a [`Vec<u16>`] as well as the position of the nul value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = U16CString::from_str(s).unwrap();
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    ///
    /// The following example demonstrates errors from nul values in a string.
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let res = U16CString::from_str(s);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 2);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str(s: impl AsRef<str>) -> Result<Self, ContainsNul<u16>> {
        let v: Vec<u16> = s.as_ref().encode_utf16().collect();
        Self::from_vec(v)
    }

    /// Encodes a [`U16CString`] copied from a [`str`], without checking for interior nul values.
    ///
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_str`][Self::from_str] except that no runtime assertion
    /// is made that `s` contains no interior nul values. Providing a string with nul values that
    /// are not the last character will result in an invalid [`U16CString`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = unsafe { U16CString::from_str_unchecked(s) };
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    #[inline]
    pub unsafe fn from_str_unchecked(s: impl AsRef<str>) -> Self {
        let v: Vec<u16> = s.as_ref().encode_utf16().collect();
        Self::from_vec_unchecked(v)
    }

    /// Encodes a [`U16CString`] copied from a [`str`], truncating at the first nul terminator.
    ///
    /// The string will be truncated at the first nul value in the string.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let wcstr = U16CString::from_str_truncate(s);
    /// assert_eq!(wcstr.to_string_lossy(), "My");
    /// ```
    #[inline]
    pub fn from_str_truncate(s: impl AsRef<str>) -> Self {
        let v: Vec<u16> = s.as_ref().encode_utf16().collect();
        Self::from_vec_truncate(v)
    }

    /// Encodes a [`U16CString`] copied from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr].
    ///
    /// The string will be scanned for nul values, which are invalid anywhere except the final
    /// character.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms (such as
    /// windows) no changes to the string will be made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value anywhere except the
    /// last character.
    /// The returned error will contain a [`Vec<u16>`] as well as the position of the nul value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = U16CString::from_os_str(s).unwrap();
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    ///
    /// The following example demonstrates errors from nul values in the string.
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let res = U16CString::from_os_str(s);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 2);
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn from_os_str(s: impl AsRef<std::ffi::OsStr>) -> Result<Self, ContainsNul<u16>> {
        let v = crate::platform::os_to_wide(s.as_ref());
        Self::from_vec(v)
    }

    /// Encodes a [`U16CString`] from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr], without checking for nul values.
    ///
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms (such as
    /// windows) no changes to the string will be made.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_os_str`][Self::from_os_str] except that no runtime
    /// assertion is made that `s` contains no interior nul values. Providing a string with nul
    /// values anywhere but the last character will result in an invalid [`U16CString`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = unsafe { U16CString::from_os_str_unchecked(s) };
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub unsafe fn from_os_str_unchecked(s: impl AsRef<std::ffi::OsStr>) -> Self {
        let v = crate::platform::os_to_wide(s.as_ref());
        Self::from_vec_unchecked(v)
    }

    /// Encodes a [`U16CString`] copied from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr], truncating at the first nul terminator.
    ///
    /// The string will be truncated at the first nul value in the string.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms (such as
    /// windows) no changes to the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let wcstr = U16CString::from_os_str_truncate(s);
    /// assert_eq!(wcstr.to_string_lossy(), "My");
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn from_os_str_truncate(s: impl AsRef<std::ffi::OsStr>) -> Self {
        let v = crate::platform::os_to_wide(s.as_ref());
        Self::from_vec_truncate(v)
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_str_truncate` instead"]
    pub fn from_str_with_nul(s: impl AsRef<str>) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_str_truncate(s))
    }

    #[cfg(feature = "std")]
    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_os_str_truncate` instead"]
    pub fn from_os_str_with_nul(
        s: impl AsRef<std::ffi::OsStr>,
    ) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_os_str_truncate(s))
    }

    #[doc(hidden)]
    #[deprecated = "use `from_str_unchecked` instead"]
    pub unsafe fn from_str_with_nul_unchecked(s: impl AsRef<str>) -> Self {
        Self::from_str_unchecked(s)
    }

    #[cfg(feature = "std")]
    #[doc(hidden)]
    #[deprecated = "use `from_os_str_unchecked` instead"]
    pub unsafe fn from_os_str_with_nul_unchecked(s: impl AsRef<std::ffi::OsStr>) -> Self {
        Self::from_os_str_unchecked(s)
    }
}

impl UCString<u32> {
    /// Constructs a [`U32CString`] from a container of character data, checking for invalid nul
    /// values.
    ///
    /// This method will consume the provided data and use the underlying elements to construct a
    /// new string. The data will be scanned for invalid nul values anywhere except the last
    /// character.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value anywhere except the
    /// last character.
    /// The returned error will contain the [`Vec<u32>`] as well as the position of the nul value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v: Vec<char> = "Test".chars().collect();
    /// # let cloned: Vec<u32> = v.iter().map(|&c| c as u32).collect();
    /// // Create a wide string from the vector
    /// let wcstr = U32CString::from_chars(v).unwrap();
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    ///
    /// The following example demonstrates errors from nul values in a vector.
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v: Vec<char> = "T\u{0}est".chars().collect();
    /// // Create a wide string from the vector
    /// let res = U32CString::from_chars(v);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 1);
    /// ```
    pub fn from_chars(v: impl Into<Vec<char>>) -> Result<Self, ContainsNul<u32>> {
        let mut chars = v.into();
        let v: Vec<u32> = unsafe {
            let ptr = chars.as_mut_ptr() as *mut u32;
            let len = chars.len();
            let cap = chars.capacity();
            mem::forget(chars);
            Vec::from_raw_parts(ptr, len, cap)
        };
        Self::from_vec(v)
    }

    /// Constructs a [`U32CString`] from a container of character data, truncating at the first nul
    /// value.
    ///
    /// This method will consume the provided data and use the underlying elements to construct a
    /// new string. The string will be truncated at the first nul value in the string.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let v: Vec<char> = "Test\u{0}".chars().collect();
    /// # let cloned: Vec<u32> = v[..4].iter().map(|&c| c as u32).collect();
    /// // Create a wide string from the vector
    /// let wcstr = U32CString::from_chars_truncate(v);
    /// # assert_eq!(wcstr.into_vec(), cloned);
    /// ```
    pub fn from_chars_truncate(v: impl Into<Vec<char>>) -> Self {
        let mut chars = v.into();
        let v: Vec<u32> = unsafe {
            let ptr = chars.as_mut_ptr() as *mut u32;
            let len = chars.len();
            let cap = chars.capacity();
            mem::forget(chars);
            Vec::from_raw_parts(ptr, len, cap)
        };
        Self::from_vec_truncate(v)
    }

    /// Constructs a [`U32CString`] from character data without checking for nul values.
    ///
    /// A terminating nul value will be appended if the vector does not already have a terminating
    /// nul.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_chars`][Self::from_chars] except that no runtime
    /// assertion is made that `v` contains no interior nul values. Providing a vector with nul
    /// values anywhere but the last character will result in an invalid [`U32CString`].
    pub unsafe fn from_chars_unchecked(v: impl Into<Vec<char>>) -> Self {
        let mut chars = v.into();
        let v: Vec<u32> = {
            let ptr = chars.as_mut_ptr() as *mut u32;
            let len = chars.len();
            let cap = chars.capacity();
            mem::forget(chars);
            Vec::from_raw_parts(ptr, len, cap)
        };
        Self::from_vec_unchecked(v)
    }

    /// Encodes a [`U32CString`] copied from a [`str`], checking for invalid interior nul values.
    ///
    /// The string will be scanned for nul values, which are invalid anywhere except the last
    /// character.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value anywhere except the
    /// last character.
    /// The returned error will contain a [`Vec<u32>`] as well as the position of the nul value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = U32CString::from_str(s).unwrap();
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    ///
    /// The following example demonstrates errors from nul values in a string.
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let res = U32CString::from_str(s);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 2);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str(s: impl AsRef<str>) -> Result<Self, ContainsNul<u32>> {
        let v: Vec<char> = s.as_ref().chars().collect();
        Self::from_chars(v)
    }

    /// Encodes a [`U32CString`] copied from a [`str`], without checking for nul values.
    ///
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_str`][Self::from_str] except that no runtime assertion
    /// is made that `s` contains invalid nul values. Providing a string with nul values anywhere
    /// except the last character will result in an invalid [`U32CString`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = unsafe { U32CString::from_str_unchecked(s) };
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    #[inline]
    pub unsafe fn from_str_unchecked(s: impl AsRef<str>) -> Self {
        let v: Vec<char> = s.as_ref().chars().collect();
        Self::from_chars_unchecked(v)
    }

    /// Encodes a [`U32CString`] copied from a [`str`], truncating at the first nul terminator.
    ///
    /// The string will be truncated at the first nul value in the string.
    /// The resulting string will always be nul-terminated even if the original string is not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let wcstr = U32CString::from_str_truncate(s);
    /// assert_eq!(wcstr.to_string_lossy(), "My");
    /// ```
    #[inline]
    pub fn from_str_truncate(s: impl AsRef<str>) -> Self {
        let v: Vec<char> = s.as_ref().chars().collect();
        Self::from_chars_truncate(v)
    }

    /// Constructs a new [`UCString`] copied from a nul-terminated [`char`] string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// If you wish to avoid copying the string pointer, use [`UCStr::from_char_ptr_str`] instead.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// # Panics
    ///
    /// This function panics if `p` is null.
    ///
    /// # Caveat
    ///
    /// The lifetime for the returned string is inferred from its usage. To prevent accidental
    /// misuse, it's suggested to tie the lifetime to whichever source lifetime is safe in the
    /// context, such as by providing a helper function taking the lifetime of a host value for the
    /// string, or by explicit annotation.
    #[inline]
    pub unsafe fn from_char_ptr_str(p: *const char) -> Self {
        Self::from_ptr_str(p as *const u32)
    }

    /// Constructs a [`UCString`] copied from a [`char`] pointer and a length, checking for invalid
    /// interior nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. If `len` is `0`, `p` is allowed to be a
    /// null pointer.
    ///
    /// The resulting string will always be nul-terminated even if the pointer data is not.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found. To
    /// avoid scanning for interior nuls, [`from_ptr_unchecked`][Self::from_ptr_unchecked] may be
    /// used instead.
    /// The returned error will contain a [`Vec`] as well as the position of the nul value.
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
    pub unsafe fn from_char_ptr(p: *const char, len: usize) -> Result<Self, ContainsNul<u32>> {
        Self::from_ptr(p as *const u32, len)
    }

    /// Constructs a [`UCString`] copied from a [`char`] pointer and a length, truncating at the
    /// first nul terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`. If no
    /// nul value is found, the whole string of length `len` is used, and a new nul-terminator
    /// will be added to the resulting string. If `len` is `0`, `p` is allowed to be a null pointer.
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
    pub unsafe fn from_char_ptr_truncate(p: *const char, len: usize) -> Self {
        Self::from_ptr_truncate(p as *const u32, len)
    }

    /// Constructs a [`UCString`] copied from a [`char`] pointer and a length without checking for
    /// any nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. If `len` is `0`, `p` is allowed to be a
    /// null pointer.
    ///
    /// The resulting string will always be nul-terminated even if the pointer data is not.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len`
    /// elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// will result in an invalid [`UCString`].
    ///
    /// # Panics
    ///
    /// Panics if `len` is greater than 0 but `p` is a null pointer.
    pub unsafe fn from_char_ptr_unchecked(p: *const char, len: usize) -> Self {
        Self::from_ptr_unchecked(p as *const u32, len)
    }

    /// Encodes a [`U32CString`] copied from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr], checking for invalid nul values.
    ///
    /// The string will be scanned for nul values, which are invlaid anywhere except the last
    /// character.
    /// The resulting string will always be nul-terminated even if the string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms no changes to
    /// the string will be made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data contains a nul value anywhere except the
    /// last character.
    /// The returned error will contain a [`Vec<u16>`] as well as the position of the nul value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = U32CString::from_os_str(s).unwrap();
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    ///
    /// The following example demonstrates errors from nul values in a string.
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let res = U32CString::from_os_str(s);
    /// assert!(res.is_err());
    /// assert_eq!(res.err().unwrap().nul_position(), 2);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub fn from_os_str(s: impl AsRef<std::ffi::OsStr>) -> Result<Self, ContainsNul<u32>> {
        let v: Vec<char> = s.as_ref().to_string_lossy().chars().collect();
        UCString::from_chars(v)
    }

    /// Encodes a [`U32CString`] copied from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr], without checking for nul values.
    ///
    /// The resulting string will always be nul-terminated even if the string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms no changes to
    /// the string will be made.
    ///
    /// # Safety
    ///
    /// This method is equivalent to [`from_os_str`][Self::from_os_str] except that no runtime
    /// assertion is made that `s` contains invalid nul values. Providing a string with nul values
    /// anywhere except the last character will result in an invalid [`U32CString`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wcstr = unsafe { U32CString::from_os_str_unchecked(s) };
    /// # assert_eq!(wcstr.to_string_lossy(), s);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub unsafe fn from_os_str_unchecked(s: impl AsRef<std::ffi::OsStr>) -> Self {
        let v: Vec<char> = s.as_ref().to_string_lossy().chars().collect();
        UCString::from_chars_unchecked(v)
    }

    /// Encodes a [`U32CString`] copied from anything that can be converted to an
    /// [`OsStr`][std::ffi::OsStr], truncating at the first nul terminator.
    ///
    /// The string will be truncated at the first nul value in the string.
    /// The resulting string will always be nul-terminated even if the string is not.
    ///
    /// Note that the encoding of [`OsStr`][std::ffi::OsStr] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms no changes to
    /// the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "My\u{0}String";
    /// // Create a wide string from the string
    /// let wcstr = U32CString::from_os_str_truncate(s);
    /// assert_eq!(wcstr.to_string_lossy(), "My");
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub fn from_os_str_truncate(s: impl AsRef<std::ffi::OsStr>) -> Self {
        let v: Vec<char> = s.as_ref().to_string_lossy().chars().collect();
        UCString::from_chars_truncate(v)
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_str_truncate` instead"]
    pub fn from_str_with_nul(s: impl AsRef<str>) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_str_truncate(s))
    }

    #[cfg(feature = "std")]
    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_os_str_truncate` instead"]
    pub fn from_os_str_with_nul(
        s: impl AsRef<std::ffi::OsStr>,
    ) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_os_str_truncate(s))
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_chars_truncate` instead"]
    pub fn from_chars_with_nul(v: impl Into<Vec<char>>) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_chars_truncate(v))
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_char_ptr_truncate` instead"]
    pub unsafe fn from_char_ptr_with_nul(
        p: *const char,
        len: usize,
    ) -> Result<Self, crate::MissingNulError> {
        Ok(Self::from_char_ptr_truncate(p, len))
    }

    #[doc(hidden)]
    #[deprecated = "use `from_str_unchecked` instead"]
    pub unsafe fn from_str_with_nul_unchecked(s: impl AsRef<str>) -> Self {
        Self::from_str_unchecked(s)
    }

    #[cfg(feature = "std")]
    #[doc(hidden)]
    #[deprecated = "use `from_os_str_unchecked` instead"]
    pub unsafe fn from_os_str_with_nul_unchecked(s: impl AsRef<std::ffi::OsStr>) -> Self {
        Self::from_os_str_unchecked(s)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_chars_unchecked` instead"]
    pub unsafe fn from_chars_with_nul_unchecked(v: impl Into<Vec<char>>) -> Self {
        Self::from_chars_unchecked(v)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_char_ptr_unchecked` instead"]
    pub unsafe fn from_char_ptr_with_nul_unchecked(p: *const char, len: usize) -> Self {
        Self::from_char_ptr_unchecked(p, len)
    }
}

impl<C: UChar> AsMut<UCStr<C>> for UCString<C> {
    fn as_mut(&mut self) -> &mut UCStr<C> {
        self.as_mut_ucstr()
    }
}

impl<C: UChar> AsRef<UCStr<C>> for UCString<C> {
    #[inline]
    fn as_ref(&self) -> &UCStr<C> {
        self.as_ucstr()
    }
}

impl<C: UChar> AsRef<[C]> for UCString<C> {
    #[inline]
    fn as_ref(&self) -> &[C] {
        self.as_slice()
    }
}

impl<C: UChar> AsRef<UStr<C>> for UCString<C> {
    #[inline]
    fn as_ref(&self) -> &UStr<C> {
        self.as_ustr()
    }
}

impl<C: UChar> Borrow<UCStr<C>> for UCString<C> {
    #[inline]
    fn borrow(&self) -> &UCStr<C> {
        self.as_ucstr()
    }
}

impl<C: UChar> BorrowMut<UCStr<C>> for UCString<C> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut UCStr<C> {
        self.as_mut_ucstr()
    }
}

impl core::fmt::Debug for U16CString {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u16(self.as_slice_with_nul(), f)
    }
}

impl core::fmt::Debug for U32CString {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u32(self.as_slice_with_nul(), f)
    }
}

impl<C: UChar> Default for UCString<C> {
    #[inline]
    fn default() -> Self {
        unsafe { Self::from_vec_unchecked(Vec::new()) }
    }
}

impl<C: UChar> Deref for UCString<C> {
    type Target = UCStr<C>;

    #[inline]
    fn deref(&self) -> &UCStr<C> {
        self.as_ucstr()
    }
}

impl<C: UChar> DerefMut for UCString<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_ucstr()
    }
}

// Turns this `UCString` into an empty string to prevent
// memory unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl<C: UChar> Drop for UCString<C> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.inner.get_unchecked_mut(0) = UChar::NUL;
        }
    }
}

impl<C: UChar> From<UCString<C>> for Vec<C> {
    #[inline]
    fn from(value: UCString<C>) -> Self {
        value.into_vec()
    }
}

impl<'a, C: UChar> From<UCString<C>> for Cow<'a, UCStr<C>> {
    #[inline]
    fn from(s: UCString<C>) -> Cow<'a, UCStr<C>> {
        Cow::Owned(s)
    }
}

#[cfg(feature = "std")]
impl From<UCString<u16>> for std::ffi::OsString {
    #[inline]
    fn from(s: UCString<u16>) -> std::ffi::OsString {
        s.to_os_string()
    }
}

#[cfg(feature = "std")]
impl From<UCString<u32>> for std::ffi::OsString {
    #[inline]
    fn from(s: UCString<u32>) -> std::ffi::OsString {
        s.to_os_string()
    }
}

impl<C: UChar> From<UCString<C>> for UString<C> {
    #[inline]
    fn from(s: UCString<C>) -> Self {
        s.to_ustring()
    }
}

impl<'a, C: UChar, T: ?Sized + AsRef<UCStr<C>>> From<&'a T> for UCString<C> {
    #[inline]
    fn from(s: &'a T) -> Self {
        s.as_ref().to_ucstring()
    }
}

impl<'a> From<&'a UCStr<u16>> for Cow<'a, UCStr<u16>> {
    #[inline]
    fn from(s: &'a UCStr<u16>) -> Cow<'a, UCStr<u16>> {
        Cow::Borrowed(s)
    }
}

impl<'a> From<&'a UCStr<u32>> for Cow<'a, UCStr<u32>> {
    #[inline]
    fn from(s: &'a UCStr<u32>) -> Cow<'a, UCStr<u32>> {
        Cow::Borrowed(s)
    }
}

impl<C: UChar> From<Box<UCStr<C>>> for UCString<C> {
    #[inline]
    fn from(s: Box<UCStr<C>>) -> Self {
        s.into_ucstring()
    }
}

impl<C: UChar> From<UCString<C>> for Box<UCStr<C>> {
    #[inline]
    fn from(s: UCString<C>) -> Box<UCStr<C>> {
        s.into_boxed_ucstr()
    }
}

impl<C: UChar> Index<RangeFull> for UCString<C> {
    type Output = UCStr<C>;

    #[inline]
    fn index(&self, _index: RangeFull) -> &UCStr<C> {
        self.as_ucstr()
    }
}

impl<C: UChar> IndexMut<RangeFull> for UCString<C> {
    #[inline]
    fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output {
        self.as_mut_ucstr()
    }
}

impl<C: UChar> PartialEq<UStr<C>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &UStr<C>) -> bool {
        self.as_ucstr() == other
    }
}

impl<C: UChar> PartialEq<UCStr<C>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &UCStr<C>) -> bool {
        self.as_ucstr() == other
    }
}

impl<'a, C: UChar> PartialEq<&'a UStr<C>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &&'a UStr<C>) -> bool {
        self.as_ucstr() == *other
    }
}

impl<'a, C: UChar> PartialEq<&'a UCStr<C>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &&'a UCStr<C>) -> bool {
        self.as_ucstr() == *other
    }
}

impl<'a, C: UChar> PartialEq<Cow<'a, UStr<C>>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &Cow<'a, UStr<C>>) -> bool {
        self.as_ucstr() == other.as_ref()
    }
}

impl<'a, C: UChar> PartialEq<Cow<'a, UCStr<C>>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &Cow<'a, UCStr<C>>) -> bool {
        self.as_ucstr() == other.as_ref()
    }
}

impl<C: UChar> PartialEq<UString<C>> for UCString<C> {
    #[inline]
    fn eq(&self, other: &UString<C>) -> bool {
        self.as_ustr() == other.as_ustr()
    }
}

impl<C: UChar> PartialOrd<UStr<C>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UStr<C>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(other)
    }
}

impl<C: UChar> PartialOrd<UCStr<C>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UCStr<C>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(other)
    }
}

impl<'a, C: UChar> PartialOrd<&'a UStr<C>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &&'a UStr<C>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(*other)
    }
}

impl<'a, C: UChar> PartialOrd<&'a UCStr<C>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &&'a UCStr<C>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(*other)
    }
}

impl<'a, C: UChar> PartialOrd<Cow<'a, UStr<C>>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &Cow<'a, UStr<C>>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(other.as_ref())
    }
}

impl<'a, C: UChar> PartialOrd<Cow<'a, UCStr<C>>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &Cow<'a, UCStr<C>>) -> Option<cmp::Ordering> {
        self.as_ucstr().partial_cmp(other.as_ref())
    }
}

impl<C: UChar> PartialOrd<UString<C>> for UCString<C> {
    #[inline]
    fn partial_cmp(&self, other: &UString<C>) -> Option<cmp::Ordering> {
        self.as_ustr().partial_cmp(other.as_ustr())
    }
}

impl<C: UChar> ToOwned for UCStr<C> {
    type Owned = UCString<C>;

    #[inline]
    fn to_owned(&self) -> UCString<C> {
        self.to_ucstring()
    }
}

/// An owned, mutable C-style "wide" string for FFI that is nul-aware and nul-terminated.
///
/// `U16CString` is aware of nul values. Unless unchecked conversions are used, all `U16CString`
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-16 data. These strings are intended to
/// be used with FFI functions such as Windows API that may require nul-terminated strings.
///
/// `U16CString` can be converted to and from many other string types, including `U16String`,
/// `OsString`, and `String`, making proper Unicode FFI safe and easy.
///
/// # Examples
///
/// The following example constructs a `U16CString` and shows how to convert a `U16CString` to a
/// regular Rust `String`.
///
/// ```rust
/// use widestring::U16CString;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U16CString::from_str(s).unwrap();
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
pub type U16CString = UCString<u16>;

/// An owned, mutable C-style wide string for FFI that is nul-aware and nul-terminated.
///
/// `U32CString` is aware of nul values. Unless unchecked conversions are used, all `U32CString`
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-32 data. These strings are intended to
/// be used with FFI functions such as Windows API that may require nul-terminated strings.
///
/// `U32CString` can be converted to and from many other string types, including `U32String`,
/// `OsString`, and `String`, making proper Unicode FFI safe and easy.
///
/// # Examples
///
/// The following example constructs a `U32CString` and shows how to convert a `U32CString` to a
/// regular Rust `String`.
///
/// ```rust
/// use widestring::U32CString;
/// let s = "Test";
/// // Create a wide string from the rust string
/// let wstr = U32CString::from_str(s).unwrap();
/// // Convert back to a rust string
/// let rust_str = wstr.to_string_lossy();
/// assert_eq!(rust_str, "Test");
/// ```
pub type U32CString = UCString<u32>;

/// Alias for `U16String` or `U32String` depending on platform. Intended to match typical C
/// `wchar_t` size on platform.
pub type WideCString = UCString<WideChar>;
