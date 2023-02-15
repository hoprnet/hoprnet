//! C-style wide string slices.
//!
//! This module contains the [`UCStr`] string slices and related types.

use crate::{
    error::{ContainsNul, MissingNulTerminator, NulError},
    iter::{CharsLossy, Utf16CharIndices, Utf16CharIndicesLossy, Utf16Chars, Utf32Chars},
    UChar, UStr, WideChar,
};
#[cfg(feature = "alloc")]
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{FromUtf16Error, String},
};
use core::{fmt::Write, ops::Range, slice};

/// C-style wide string reference for [`UCString`][crate::UCString].
///
/// [`UCStr`] is aware of nul values. Unless unchecked conversions are used, all [`UCStr`]
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-16 or UTF-32 data. These strings are
/// intended to be used with FFI functions such as Windows API that may require nul-terminated
/// strings.
///
/// [`UCStr`] can be converted to and from many other string types, including
/// [`UString`][crate::UString], [`OsString`][std::ffi::OsString], and [`String`], making proper
/// Unicode FFI safe and easy.
///
/// Please prefer using the type aliases [`U16CStr`], [`U32CStr`], or [`WideCStr`] to using
/// [`UCStr`] directly.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UCStr<C> {
    inner: [C],
}

impl<C: UChar> UCStr<C> {
    /// Coerces a value into a [`UCStr`].
    #[inline]
    pub fn new<S: AsRef<UCStr<C>> + ?Sized>(s: &S) -> &Self {
        s.as_ref()
    }

    /// Constructs a [`UCStr`] from a nul-terminated string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_ptr_str<'a>(p: *const C) -> &'a Self {
        assert!(!p.is_null());
        let mut i = 0;
        while *p.add(i) != UChar::NUL {
            i += 1;
        }
        Self::from_ptr_unchecked(p, i)
    }

    /// Constructs a mutable [`UCStr`] from a mutable nul-terminated string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_ptr_str_mut<'a>(p: *mut C) -> &'a mut Self {
        assert!(!p.is_null());
        let mut i = 0;
        while *p.add(i) != UChar::NUL {
            i += 1;
        }
        Self::from_ptr_unchecked_mut(p, i)
    }

    /// Constructs a [`UCStr`] from a pointer and a length.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found
    /// before the nul terminator at `len` offset. To avoid scanning for interior nuls,
    /// [`from_ptr_unchecked`][Self::from_ptr_unchecked] may be used instead.
    ///
    /// An error is returned if the value at `len` offset is not a nul terminator.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_ptr<'a>(p: *const C, len: usize) -> Result<&'a Self, NulError<C>> {
        assert!(!p.is_null());
        if *p.add(len) != UChar::NUL {
            return Err(MissingNulTerminator::new().into());
        }
        for i in 0..len {
            if *p.add(i) == UChar::NUL {
                return Err(ContainsNul::empty(i).into());
            }
        }
        Ok(Self::from_ptr_unchecked(p, len))
    }

    /// Constructs a mutable [`UCStr`] from a mutable pointer and a length.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found
    /// before the nul terminator at `len` offset. To avoid scanning for interior nuls,
    /// [`from_ptr_unchecked_mut`][Self::from_ptr_unchecked_mut] may be used instead.
    ///
    /// An error is returned if the value at `len` offset is not a nul terminator.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_ptr_mut<'a>(p: *mut C, len: usize) -> Result<&'a mut Self, NulError<C>> {
        assert!(!p.is_null());
        if *p.add(len) != UChar::NUL {
            return Err(MissingNulTerminator::new().into());
        }
        for i in 0..len {
            if *p.add(i) == UChar::NUL {
                return Err(ContainsNul::empty(i).into());
            }
        }
        Ok(Self::from_ptr_unchecked_mut(p, len))
    }

    /// Constructs a [`UCStr`] from a pointer and a length, truncating at the first nul terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`.
    ///
    /// # Errors
    ///
    /// If no nul terminator is found after `len` + 1 elements, an error is returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_ptr_truncate<'a>(
        p: *const C,
        len: usize,
    ) -> Result<&'a Self, MissingNulTerminator> {
        assert!(!p.is_null());
        for i in 0..=len {
            if *p.add(i) == UChar::NUL {
                return Ok(Self::from_ptr_unchecked(p, i));
            }
        }
        Err(MissingNulTerminator::new())
    }

    /// Constructs a mutable [`UCStr`] from a mutable pointer and a length, truncating at the first
    /// nul terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`.
    ///
    /// # Errors
    ///
    /// If no nul terminator is found after `len` + 1 elements, an error is returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_ptr_truncate_mut<'a>(
        p: *mut C,
        len: usize,
    ) -> Result<&'a mut Self, MissingNulTerminator> {
        assert!(!p.is_null());
        for i in 0..=len {
            if *p.add(i) == UChar::NUL {
                return Ok(Self::from_ptr_unchecked_mut(p, i));
            }
        }
        Err(MissingNulTerminator::new())
    }

    /// Constructs a [`UCStr`] from a pointer and a length without checking for any nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements, nor that it has a terminating nul value.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// a missing nul terminator at pointer offset `len` + 1 will result in an invalid [`UCStr`].
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
    pub unsafe fn from_ptr_unchecked<'a>(p: *const C, len: usize) -> &'a Self {
        assert!(!p.is_null());
        let ptr: *const [C] = slice::from_raw_parts(p, len + 1);
        &*(ptr as *const Self)
    }

    /// Constructs a mutable [`UCStr`] from a mutable pointer and a length without checking for any
    /// nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements, nor that is has a terminating nul value.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// a missing nul terminator at pointer offset `len` + 1 will result in an invalid [`UCStr`].
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
    pub unsafe fn from_ptr_unchecked_mut<'a>(p: *mut C, len: usize) -> &'a mut Self {
        assert!(!p.is_null());
        let ptr: *mut [C] = slice::from_raw_parts_mut(p, len + 1);
        &mut *(ptr as *mut Self)
    }

    /// Constructs a [`UCStr`] from a slice of values with a terminating nul, checking for invalid
    /// interior nul values.
    ///
    /// The slice must have at least one item, the nul terminator, even for an empty string.
    ///
    /// # Errors
    ///
    /// If there are nul values in the slice except for the last value, an error is returned.
    ///
    /// An error is also returned if the last value of the slice is not a nul terminator.
    pub fn from_slice(slice: &[C]) -> Result<&Self, NulError<C>> {
        if slice.last() != Some(&UChar::NUL) {
            return Err(MissingNulTerminator::new().into());
        }
        match slice[..slice.len() - 1]
            .iter()
            .position(|x| *x == UChar::NUL)
        {
            None => Ok(unsafe { Self::from_slice_unchecked(slice) }),
            Some(i) => Err(ContainsNul::empty(i).into()),
        }
    }

    /// Constructs a mutable [`UCStr`] from a mutable slice of values with a terminating nul,
    /// checking for invalid interior nul values.
    ///
    /// The slice must have at least one item, the nul terminator, even for an empty string.
    ///
    /// # Errors
    ///
    /// If there are nul values in the slice except for the last value, an error is returned.
    ///
    /// An error is also returned if the last value of the slice is not a nul terminator.
    pub fn from_slice_mut(slice: &mut [C]) -> Result<&mut Self, NulError<C>> {
        if slice.last() != Some(&UChar::NUL) {
            return Err(MissingNulTerminator::new().into());
        }
        match slice[..slice.len() - 1]
            .iter()
            .position(|x| *x == UChar::NUL)
        {
            None => Ok(unsafe { Self::from_slice_unchecked_mut(slice) }),
            Some(i) => Err(ContainsNul::empty(i).into()),
        }
    }

    /// Constructs a [`UCStr`] from a slice of values, truncating at the first nul terminator.
    ///
    /// The slice will be scanned for nul values. When a nul value is found, it is treated as the
    /// terminator for the string, and the [`UCStr`] slice will be truncated to that nul.
    ///
    /// # Errors
    ///
    /// If there are no nul values in the slice, an error is returned.
    pub fn from_slice_truncate(slice: &[C]) -> Result<&Self, MissingNulTerminator> {
        match slice.iter().position(|x| *x == UChar::NUL) {
            None => Err(MissingNulTerminator::new()),
            Some(i) => Ok(unsafe { Self::from_slice_unchecked(&slice[..i + 1]) }),
        }
    }

    /// Constructs a mutable [`UCStr`] from a mutable slice of values, truncating at the first nul
    /// terminator.
    ///
    /// The slice will be scanned for nul values. When a nul value is found, it is treated as the
    /// terminator for the string, and the [`UCStr`] slice will be truncated to that nul.
    ///
    /// # Errors
    ///
    /// If there are no nul values in the slice, an error is returned.
    pub fn from_slice_truncate_mut(slice: &mut [C]) -> Result<&mut Self, MissingNulTerminator> {
        match slice.iter().position(|x| *x == UChar::NUL) {
            None => Err(MissingNulTerminator::new()),
            Some(i) => Ok(unsafe { Self::from_slice_unchecked_mut(&mut slice[..i + 1]) }),
        }
    }

    /// Constructs a [`UCStr`] from a slice of values without checking for a terminating or interior
    /// nul values.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can lead to invalid [`UCStr`] values when the slice
    /// is missing a terminating nul value or there are non-terminating interior nul values
    /// in the slice. In particular, an empty slice will result in an invalid [`UCStr`].
    pub unsafe fn from_slice_unchecked(slice: &[C]) -> &Self {
        let ptr: *const [C] = slice;
        &*(ptr as *const Self)
    }

    /// Constructs a mutable [`UCStr`] from a mutable slice of values without checking for a
    /// terminating or interior nul values.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can lead to invalid [`UCStr`] values when the slice
    /// is missing a terminating nul value or there are non-terminating interior nul values
    /// in the slice. In particular, an empty slice will result in an invalid [`UCStr`].
    pub unsafe fn from_slice_unchecked_mut(slice: &mut [C]) -> &mut Self {
        let ptr: *mut [C] = slice;
        &mut *(ptr as *mut Self)
    }

    /// Copies the string reference to a new owned [`UCString`][crate::UCString].
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_ucstring(&self) -> crate::UCString<C> {
        unsafe { crate::UCString::from_vec_unchecked(self.inner.to_owned()) }
    }

    /// Copies the string reference to a new owned [`UString`][crate::UString].
    ///
    /// The resulting [`UString`][crate::UString] will **not** have a nul terminator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let wcstr = U16CString::from_str("MyString").unwrap();
    /// // Convert U16CString to a U16String
    /// let wstr = wcstr.to_ustring();
    ///
    /// // U16CString will have a terminating nul
    /// let wcvec = wcstr.into_vec_with_nul();
    /// assert_eq!(wcvec[wcvec.len()-1], 0);
    /// // The resulting U16String will not have the terminating nul
    /// let wvec = wstr.into_vec();
    /// assert_ne!(wvec[wvec.len()-1], 0);
    /// ```
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let wcstr = U32CString::from_str("MyString").unwrap();
    /// // Convert U32CString to a U32String
    /// let wstr = wcstr.to_ustring();
    ///
    /// // U32CString will have a terminating nul
    /// let wcvec = wcstr.into_vec_with_nul();
    /// assert_eq!(wcvec[wcvec.len()-1], 0);
    /// // The resulting U32String will not have the terminating nul
    /// let wvec = wstr.into_vec();
    /// assert_ne!(wvec[wvec.len()-1], 0);
    /// ```
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_ustring(&self) -> crate::UString<C> {
        crate::UString::from_vec(self.as_slice())
    }

    /// Converts to a slice of the underlying code units.
    ///
    /// The slice will **not** include the nul terminator.
    #[inline]
    pub fn as_slice(&self) -> &[C] {
        &self.inner[..self.len()]
    }

    /// Converts to a mutable slice of the underlying code units.
    ///
    /// The slice will **not** include the nul terminator.
    ///
    /// # Safety
    ///
    /// This method is unsafe because you can violate the invariants of this type when mutating the
    /// slice (i.e. by adding interior nul values).
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [C] {
        let len = self.len();
        &mut self.inner[..len]
    }

    /// Converts to a slice of the underlying code units, including the nul terminator.
    #[inline]
    pub fn as_slice_with_nul(&self) -> &[C] {
        &self.inner
    }

    /// Returns a raw pointer to the string.
    ///
    /// The caller must ensure that the string outlives the pointer this function returns, or else
    /// it will end up pointing to garbage.
    ///
    /// The caller must also ensure that the memory the pointer (non-transitively) points to is
    /// never written to (except inside an `UnsafeCell`) using this pointer or any pointer derived
    /// from it. If you need to mutate the contents of the string, use
    /// [`as_mut_ptr`][Self::as_mut_ptr].
    ///
    /// Modifying the container referenced by this string may cause its buffer to be reallocated,
    /// which would also make any pointers to it invalid.
    #[inline]
    pub fn as_ptr(&self) -> *const C {
        self.inner.as_ptr()
    }

    /// Returns a mutable raw pointer to the string.
    ///
    /// The caller must ensure that the string outlives the pointer this function returns, or else
    /// it will end up pointing to garbage.
    ///
    /// Modifying the container referenced by this string may cause its buffer to be reallocated,
    /// which would also make any pointers to it invalid.
    ///
    /// # Safety
    ///
    /// This method is unsafe because you can violate the invariants of this type when mutating the
    /// memory the pointer points to (i.e. by adding interior nul values).
    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut C {
        self.inner.as_mut_ptr()
    }

    /// Returns the two raw pointers spanning the string slice.
    ///
    /// The returned range is half-open, which means that the end pointer points one past the last
    /// element of the slice. This way, an empty slice is represented by two equal pointers, and the
    /// difference between the two pointers represents the size of the slice.
    ///
    /// See [`as_ptr`][Self::as_ptr] for warnings on using these pointers. The end pointer requires
    /// extra caution, as it does not point to a valid element in the slice.
    ///
    /// This function is useful for interacting with foreign interfaces which use two pointers to
    /// refer to a range of elements in memory, as is common in C++.
    #[inline]
    pub fn as_ptr_range(&self) -> Range<*const C> {
        self.inner.as_ptr_range()
    }

    /// Returns the two unsafe mutable pointers spanning the string slice.
    ///
    /// The returned range is half-open, which means that the end pointer points one past the last
    /// element of the slice. This way, an empty slice is represented by two equal pointers, and the
    /// difference between the two pointers represents the size of the slice.
    ///
    /// See [`as_mut_ptr`][Self::as_mut_ptr] for warnings on using these pointers. The end pointer requires
    /// extra caution, as it does not point to a valid element in the slice.
    ///
    /// This function is useful for interacting with foreign interfaces which use two pointers to
    /// refer to a range of elements in memory, as is common in C++.
    #[inline]
    pub fn as_mut_ptr_range(&mut self) -> Range<*mut C> {
        self.inner.as_mut_ptr_range()
    }

    /// Returns the length of the string as number of elements (**not** number of bytes)
    /// **not** including nul terminator.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() - 1
    }

    /// Returns whether this string contains no data (i.e. is only the nul terminator).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts a [`Box<UCStr>`] into a [`UCString`][crate::UCString] without copying or
    /// allocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use widestring::U16CString;
    ///
    /// let v = vec![102u16, 111u16, 111u16]; // "foo"
    /// let c_string = U16CString::from_vec(v.clone()).unwrap();
    /// let boxed = c_string.into_boxed_ucstr();
    /// assert_eq!(boxed.into_ucstring(), U16CString::from_vec(v).unwrap());
    /// ```
    ///
    /// ```
    /// use widestring::U32CString;
    ///
    /// let v = vec![102u32, 111u32, 111u32]; // "foo"
    /// let c_string = U32CString::from_vec(v.clone()).unwrap();
    /// let boxed = c_string.into_boxed_ucstr();
    /// assert_eq!(boxed.into_ucstring(), U32CString::from_vec(v).unwrap());
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn into_ucstring(self: Box<Self>) -> crate::UCString<C> {
        let raw = Box::into_raw(self) as *mut [C];
        crate::UCString {
            inner: unsafe { Box::from_raw(raw) },
        }
    }

    /// Returns a [`UStr`] reference to this string reference.
    ///
    /// The [`UStr`] reference will not include the nul-terminator.
    #[inline]
    pub fn as_ustr(&self) -> &UStr<C> {
        UStr::from_slice(self.as_slice())
    }

    /// Returns a [`UStr`] reference to this string reference.
    ///
    /// The [`UStr`] reference will include the nul-terminator.
    #[inline]
    pub fn as_ustr_with_nul(&self) -> &UStr<C> {
        UStr::from_slice(self.as_slice())
    }

    /// Returns a mutable [`UStr`] reference to this string reference.
    ///
    /// The [`UStr`] reference will not include the nul-terminator.
    ///
    /// # Safety
    ///
    /// This method is unsafe because you can violate the invariants of this type when mutating the
    /// string (i.e. by adding interior nul values).
    #[inline]
    pub unsafe fn as_mut_ustr(&mut self) -> &mut UStr<C> {
        UStr::from_slice_mut(self.as_mut_slice())
    }

    #[doc(hidden)]
    #[deprecated = "use `from_ptr_unchecked` instead"]
    pub unsafe fn from_ptr_with_nul<'a>(p: *const C, len: usize) -> &'a Self {
        Self::from_ptr_unchecked(p, len)
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_slice_truncate` instead"]
    pub fn from_slice_with_nul(slice: &[C]) -> Result<&Self, crate::MissingNulError> {
        Self::from_slice_truncate(slice)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_slice_unchecked` instead"]
    pub unsafe fn from_slice_with_nul_unchecked(slice: &[C]) -> &Self {
        Self::from_slice_unchecked(slice)
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn from_inner(slice: &[C]) -> &UCStr<C> {
        let ptr: *const [C] = slice;
        unsafe { &*(ptr as *const UCStr<C>) }
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn from_inner_mut(slice: &mut [C]) -> &mut UCStr<C> {
        let ptr: *mut [C] = slice;
        unsafe { &mut *(ptr as *mut UCStr<C>) }
    }

    /// Returns an object that implements [`Display`][std::fmt::Display] for printing strings that
    /// may contain non-Unicode data.
    ///
    /// A [`UCStr`] might contain ill-formed UTF encoding. This struct implements the
    /// [`Display`][std::fmt::Display] trait in a way that decoding the string is lossy but no heap
    /// allocations are performed, such as by [`to_string_lossy`][UCStr::to_string_lossy].
    ///
    /// By default, invalid Unicode data is replaced with
    /// [`U+FFFD REPLACEMENT CHARACTER`][std::char::REPLACEMENT_CHARACTER] (�). If you wish to simply
    /// skip any invalid Uncode data and forego the replacement, you may use the
    /// [alternate formatting][std::fmt#sign0] with `{:#}`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use widestring::U16CStr;
    ///
    /// // 𝄞mus<invalid>ic<invalid>
    /// let s = U16CStr::from_slice(&[
    ///     0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0xDD1E, 0x0069, 0x0063, 0xD834, 0x0000,
    /// ]).unwrap();
    ///
    /// assert_eq!(format!("{}", s.display()),
    /// "𝄞mus�ic�"
    /// );
    /// ```
    ///
    /// Using alternate formatting style to skip invalid values entirely:
    ///
    /// ```
    /// use widestring::U16CStr;
    ///
    /// // 𝄞mus<invalid>ic<invalid>
    /// let s = U16CStr::from_slice(&[
    ///     0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0xDD1E, 0x0069, 0x0063, 0xD834, 0x0000,
    /// ]).unwrap();
    ///
    /// assert_eq!(format!("{:#}", s.display()),
    /// "𝄞music"
    /// );
    /// ```
    #[inline]
    pub fn display(&self) -> Display<'_, C> {
        Display { str: self }
    }
}

impl UCStr<u16> {
    /// Decodes a string reference to an owned [`OsString`][std::ffi::OsString].
    ///
    /// This makes a string copy of the [`U16CStr`]. Since [`U16CStr`] makes no guarantees that it
    /// is valid UTF-16, there is no guarantee that the resulting [`OsString`][std::ffi::OsString]
    /// will be valid data. The [`OsString`][std::ffi::OsString] will **not** have a nul
    /// terminator.
    ///
    /// Note that the encoding of [`OsString`][std::ffi::OsString] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms (such as
    /// windows) no changes to the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// use std::ffi::OsString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U16CString::from_str(s).unwrap();
    /// // Create an OsString from the wide string
    /// let osstr = wstr.to_os_string();
    ///
    /// assert_eq!(osstr, OsString::from(s));
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn to_os_string(&self) -> std::ffi::OsString {
        crate::platform::os_from_wide(self.as_slice())
    }

    /// Decodes the string reference to a [`String`] if it contains valid UTF-16 data.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains any invalid UTF-16 data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U16CString::from_str(s).unwrap();
    /// // Create a regular string from the wide string
    /// let s2 = wstr.to_string().unwrap();
    ///
    /// assert_eq!(s2, s);
    /// ```
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_string(&self) -> Result<String, FromUtf16Error> {
        String::from_utf16(self.as_slice())
    }

    /// Decodes the string reference to a [`String`] even if it is invalid UTF-16 data.
    ///
    /// Any non-Unicode sequences are replaced with `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U16CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U16CString::from_str(s).unwrap();
    /// // Create a regular string from the wide string
    /// let s2 = wstr.to_string_lossy();
    ///
    /// assert_eq!(s2, s);
    /// ```
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_slice())
    }

    /// Returns an iterator over the [`char`][prim@char]s of a string slice.
    ///
    /// As this string slice may consist of invalid UTF-16, the iterator returned by this method
    /// is an iterator over `Result<char, DecodeUtf16Error>` instead of [`char`][prim@char]s
    /// directly. If you would like a lossy iterator over [`chars`][prim@char]s directly, instead
    /// use [`chars_lossy`][Self::chars_lossy].
    ///
    /// It's important to remember that [`char`][prim@char] represents a Unicode Scalar Value, and
    /// may not match your idea of what a 'character' is. Iteration over grapheme clusters may be
    /// what you actually want. That functionality is not provided by by this crate.
    #[inline]
    pub fn chars(&self) -> Utf16Chars<'_> {
        Utf16Chars::from_ucstr(self)
    }

    /// Returns a lossy iterator over the [`char`][prim@char]s of a string slice.
    ///
    /// As this string slice may consist of invalid UTF-16, the iterator returned by this method
    /// will replace unpaired surrogates with
    /// [`U+FFFD REPLACEMENT CHARACTER`][std::char::REPLACEMENT_CHARACTER] (�). This is a lossy
    /// version of [`chars`][Self::chars].
    ///
    /// It's important to remember that [`char`][prim@char] represents a Unicode Scalar Value, and
    /// may not match your idea of what a 'character' is. Iteration over grapheme clusters may be
    /// what you actually want. That functionality is not provided by by this crate.
    #[inline]
    pub fn chars_lossy(&self) -> CharsLossy<'_> {
        CharsLossy::from_u16cstr(self)
    }

    /// Returns an iterator over the chars of a string slice, and their positions.
    ///
    /// As this string slice may consist of invalid UTF-16, the iterator returned by this method
    /// is an iterator over `Result<char, DecodeUtf16Error>` as well as their positions, instead of
    /// [`char`][prim@char]s directly. If you would like a lossy indices iterator over
    /// [`chars`][prim@char]s directly, instead use
    /// [`char_indices_lossy`][Self::char_indices_lossy].
    ///
    /// The iterator yields tuples. The position is first, the [`char`][prim@char] is second.
    #[inline]
    pub fn char_indices(&self) -> Utf16CharIndices<'_> {
        Utf16CharIndices::from_ucstr(self)
    }

    /// Returns a lossy iterator over the chars of a string slice, and their positions.
    ///
    /// As this string slice may consist of invalid UTF-16, the iterator returned by this method
    /// will replace unpaired surrogates with
    /// [`U+FFFD REPLACEMENT CHARACTER`][std::char::REPLACEMENT_CHARACTER] (�), as well as the
    /// positions of all characters. This is a lossy version of
    /// [`char_indices`][Self::char_indices].
    ///
    /// The iterator yields tuples. The position is first, the [`char`][prim@char] is second.
    #[inline]
    pub fn char_indices_lossy(&self) -> Utf16CharIndicesLossy<'_> {
        Utf16CharIndicesLossy::from_ucstr(self)
    }
}

impl UCStr<u32> {
    /// Constructs a string reference from a [`char`] nul-terminated string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_char_ptr_str<'a>(p: *const char) -> &'a Self {
        Self::from_ptr_str(p as *const u32)
    }

    /// Constructs a mutable string reference from a mutable [`char`] nul-terminated string pointer.
    ///
    /// This will scan for nul values beginning with `p`. The first nul value will be used as the
    /// nul terminator for the string, similar to how libc string functions such as `strlen` work.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_char_ptr_str_mut<'a>(p: *mut char) -> &'a mut Self {
        Self::from_ptr_str_mut(p as *mut u32)
    }

    /// Constructs a string reference from a [`char`] pointer and a length.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found
    /// before the nul terminator at `len` offset. To avoid scanning for interior nuls,
    /// [`from_ptr_unchecked`][Self::from_ptr_unchecked] may be used instead.
    ///
    /// An error is returned if the value at `len` offset is not a nul terminator.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_char_ptr<'a>(p: *const char, len: usize) -> Result<&'a Self, NulError<u32>> {
        Self::from_ptr(p as *const u32, len)
    }

    /// Constructs a mutable string reference from a mutable [`char`] pointer and a length.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Errors
    ///
    /// This will scan the pointer string for an interior nul value and error if one is found
    /// before the nul terminator at `len` offset. To avoid scanning for interior nuls,
    /// [`from_ptr_unchecked_mut`][Self::from_ptr_unchecked_mut] may be used instead.
    ///
    /// An error is returned if the value at `len` offset is not a nul terminator.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_char_ptr_mut<'a>(
        p: *mut char,
        len: usize,
    ) -> Result<&'a mut Self, NulError<u32>> {
        Self::from_ptr_mut(p as *mut u32, len)
    }

    /// Constructs a string reference from a [`char`] pointer and a length, truncating at the first
    /// nul terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`.
    ///
    /// # Errors
    ///
    /// If no nul terminator is found after `len` + 1 elements, an error is returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
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
    pub unsafe fn from_char_ptr_truncate<'a>(
        p: *const char,
        len: usize,
    ) -> Result<&'a Self, MissingNulTerminator> {
        Self::from_ptr_truncate(p as *const u32, len)
    }

    /// Constructs a mutable string reference from a mutable [`char`] pointer and a length,
    /// truncating at the first nul terminator.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes. This will scan
    /// for nul values beginning with `p` until offset `len`. The first nul value will be used as
    /// the nul terminator for the string, ignoring any remaining values left before `len`.
    ///
    /// # Errors
    ///
    /// If no nul terminator is found after `len` + 1 elements, an error is returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid or has a
    /// nul terminator, and the function could scan past the underlying buffer.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
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
    pub unsafe fn from_char_ptr_truncate_mut<'a>(
        p: *mut char,
        len: usize,
    ) -> Result<&'a mut Self, MissingNulTerminator> {
        Self::from_ptr_truncate_mut(p as *mut u32, len)
    }

    /// Constructs a string reference from a [`char`] pointer and a length without checking for any
    /// nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements, nor that is has a terminating nul value.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts].
    /// In particular, the returned string reference *must not be mutated* for the duration of
    /// lifetime `'a`, except inside an [`UnsafeCell`][std::cell::UnsafeCell].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// a missing nul terminator at pointer offset `len` + 1 will result in an invalid [`UCStr`].
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
    pub unsafe fn from_char_ptr_unchecked<'a>(p: *const char, len: usize) -> &'a Self {
        Self::from_ptr_unchecked(p as *const u32, len)
    }

    /// Constructs a mutable string reference from a mutable [`char`] pointer and a length without
    /// checking for any nul values.
    ///
    /// The `len` argument is the number of elements, **not** the number of bytes, and does
    /// **not** include the nul terminator of the string. Thus, a `len` of 0 is valid and means
    /// that `p` is a pointer directly to the nul terminator of the string.
    ///
    /// # Safety
    ///
    /// This function is unsafe as there is no guarantee that the given pointer is valid for `len +
    /// 1` elements, nor that is has a terminating nul value.
    ///
    /// In addition, the data must meet the safety conditions of [std::slice::from_raw_parts_mut].
    ///
    /// The interior values of the pointer are not scanned for nul. Any interior nul values or
    /// a missing nul terminator at pointer offset `len` + 1 will result in an invalid [`UCStr`].
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
    pub unsafe fn from_char_ptr_unchecked_mut<'a>(p: *mut char, len: usize) -> &'a mut Self {
        Self::from_ptr_unchecked_mut(p as *mut u32, len)
    }

    /// Constructs a string reference from a [`char`] slice with a terminating nul, checking for
    /// invalid interior nul values.
    ///
    /// The slice must have at least one item, the nul terminator, even for an empty string.
    ///
    /// # Errors
    ///
    /// If there are nul values in the slice except for the last value, an error is returned.
    ///
    /// An error is also returned if the last value of the slice is not a nul terminator.
    pub fn from_char_slice(slice: &[char]) -> Result<&Self, NulError<u32>> {
        let ptr: *const [char] = slice;
        Self::from_slice(unsafe { &*(ptr as *const [u32]) })
    }

    /// Constructs a mutable string reference from a mutable [`char`] slice with a terminating nul,
    /// checking for invalid interior nul values.
    ///
    /// The slice must have at least one item, the nul terminator, even for an empty string.
    ///
    /// # Errors
    ///
    /// If there are nul values in the slice except for the last value, an error is returned.
    ///
    /// An error is also returned if the last value of the slice is not a nul terminator.
    pub fn from_char_slice_mut(slice: &mut [char]) -> Result<&mut Self, NulError<u32>> {
        let ptr: *mut [char] = slice;
        Self::from_slice_mut(unsafe { &mut *(ptr as *mut [u32]) })
    }

    /// Constructs a string reference from a slice of [`char`] values, truncating at the first nul
    /// terminator.
    ///
    /// The slice will be scanned for nul values. When a nul value is found, it is treated as the
    /// terminator for the string, and the [`UCStr`] slice will be truncated to that nul.
    ///
    /// # Errors
    ///
    /// If there are no nul values in the slice, an error is returned.
    #[inline]
    pub fn from_char_slice_truncate(slice: &[char]) -> Result<&Self, MissingNulTerminator> {
        let ptr: *const [char] = slice;
        Self::from_slice_truncate(unsafe { &*(ptr as *const [u32]) })
    }

    /// Constructs a mutable string reference from a mutable slice of [`char`] values, truncating at
    /// the first nul terminator.
    ///
    /// The slice will be scanned for nul values. When a nul value is found, it is treated as the
    /// terminator for the string, and the [`UCStr`] slice will be truncated to that nul.
    ///
    /// # Errors
    ///
    /// If there are no nul values in the slice, an error is returned.
    #[inline]
    pub fn from_char_slice_truncate_mut(
        slice: &mut [char],
    ) -> Result<&mut Self, MissingNulTerminator> {
        let ptr: *mut [char] = slice;
        Self::from_slice_truncate_mut(unsafe { &mut *(ptr as *mut [u32]) })
    }

    /// Constructs a string reference from a [`char`] slice without checking for a terminating or
    /// interior nul values.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can lead to invalid [`UCStr`] values when the slice
    /// is missing a terminating nul value or there are non-terminating interior nul values
    /// in the slice. In particular, an empty slice will result in an invalid [`UCStr`].
    #[inline]
    pub unsafe fn from_char_slice_unchecked(slice: &[char]) -> &Self {
        let ptr: *const [char] = slice;
        Self::from_slice_unchecked(&*(ptr as *const [u32]))
    }

    /// Constructs a mutable string reference from a mutable [`char`] slice without checking for a
    /// terminating or interior nul values.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can lead to invalid [`UCStr`] values when the slice
    /// is missing a terminating nul value or there are non-terminating interior nul values
    /// in the slice. In particular, an empty slice will result in an invalid [`UCStr`].
    #[inline]
    pub unsafe fn from_char_slice_unchecked_mut(slice: &mut [char]) -> &mut Self {
        let ptr: *mut [char] = slice;
        Self::from_slice_unchecked_mut(&mut *(ptr as *mut [u32]))
    }

    /// Decodes a string reference to an owned [`OsString`][std::ffi::OsString].
    ///
    /// This makes a string copy of this reference. Since [`UCStr<u32>`] makes no guarantees that it
    /// is valid UTF-32, there is no guarantee that the resulting [`OsString`][std::ffi::OsString]
    /// will be valid data. The [`OsString`][std::ffi::OsString] will **not** have a nul
    /// terminator.
    ///
    /// Note that the encoding of [`OsString`][std::ffi::OsString] is platform-dependent, so on
    /// some platforms this may make an encoding conversions, while on other platforms no changes to
    /// the string will be made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// use std::ffi::OsString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U32CString::from_str(s).unwrap();
    /// // Create an OsString from the wide string
    /// let osstr = wstr.to_os_string();
    ///
    /// assert_eq!(osstr, OsString::from(s));
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn to_os_string(&self) -> std::ffi::OsString {
        self.as_ustr().to_os_string()
    }

    /// Decodes the string reference to a [`String`] if it contains valid UTF-32 data.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains any invalid UTF-32 data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U32CString::from_str(s).unwrap();
    /// // Create a regular string from the wide string
    /// let s2 = wstr.to_string().unwrap();
    ///
    /// assert_eq!(s2, s);
    /// ```
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_string(&self) -> Result<String, crate::error::FromUtf32Error> {
        self.as_ustr().to_string()
    }

    /// Decodes the string reference to a [`String`] even if it is invalid UTF-32 data.
    ///
    /// Any non-Unicode sequences are replaced with `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use widestring::U32CString;
    /// let s = "MyString";
    /// // Create a wide string from the string
    /// let wstr = U32CString::from_str(s).unwrap();
    /// // Create a regular string from the wide string
    /// let s2 = wstr.to_string_lossy();
    ///
    /// assert_eq!(s2, s);
    /// ```
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_string_lossy(&self) -> String {
        self.as_ustr().to_string_lossy()
    }

    #[doc(hidden)]
    #[deprecated = "use `from_char_ptr_unchecked` instead"]
    pub unsafe fn from_char_ptr_with_nul<'a>(p: *const char, len: usize) -> &'a Self {
        Self::from_char_ptr_unchecked(p, len)
    }

    #[allow(deprecated)]
    #[doc(hidden)]
    #[deprecated = "use `from_char_slice_truncate` instead"]
    pub fn from_char_slice_with_nul(slice: &[char]) -> Result<&Self, crate::MissingNulError> {
        Self::from_char_slice_truncate(slice)
    }

    #[doc(hidden)]
    #[deprecated = "use `from_char_slice_unchecked` instead"]
    pub unsafe fn from_char_slice_with_nul_unchecked(slice: &[char]) -> &Self {
        Self::from_char_slice_unchecked(slice)
    }

    /// Returns an iterator over the [`char`][prim@char]s of a string slice.
    ///
    /// As this string slice may consist of invalid UTF-32, the iterator returned by this method
    /// is an iterator over `Result<char, DecodeUtf32Error>` instead of [`char`][prim@char]s
    /// directly. If you would like a lossy iterator over [`chars`][prim@char]s directly, instead
    /// use [`chars_lossy`][Self::chars_lossy].
    ///
    /// It's important to remember that [`char`][prim@char] represents a Unicode Scalar Value, and
    /// may not match your idea of what a 'character' is. Iteration over grapheme clusters may be
    /// what you actually want. That functionality is not provided by by this crate.
    #[inline]
    pub fn chars(&self) -> Utf32Chars<'_> {
        Utf32Chars::from_ucstr(self)
    }

    /// Returns a lossy iterator over the [`char`][prim@char]s of a string slice.
    ///
    /// As this string slice may consist of invalid UTF-32, the iterator returned by this method
    /// will replace surrogate values or invalid code points with
    /// [`U+FFFD REPLACEMENT CHARACTER`][std::char::REPLACEMENT_CHARACTER] (�). This is a lossy
    /// version of [`chars`][Self::chars].
    ///
    /// It's important to remember that [`char`][prim@char] represents a Unicode Scalar Value, and
    /// may not match your idea of what a 'character' is. Iteration over grapheme clusters may be
    /// what you actually want. That functionality is not provided by by this crate.
    #[inline]
    pub fn chars_lossy(&self) -> CharsLossy<'_> {
        CharsLossy::from_u32cstr(self)
    }
}

impl<C: UChar> AsMut<UCStr<C>> for UCStr<C> {
    #[inline]
    fn as_mut(&mut self) -> &mut UCStr<C> {
        self
    }
}

impl<C: UChar> AsRef<UCStr<C>> for UCStr<C> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<C: UChar> AsRef<[C]> for UCStr<C> {
    #[inline]
    fn as_ref(&self) -> &[C] {
        self.as_slice()
    }
}

impl<C: UChar> AsRef<UStr<C>> for UCStr<C> {
    #[inline]
    fn as_ref(&self) -> &UStr<C> {
        self.as_ustr()
    }
}

impl core::fmt::Debug for U16CStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u16(self.as_slice_with_nul(), f)
    }
}

impl core::fmt::Debug for U32CStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_u32(self.as_slice_with_nul(), f)
    }
}

impl<'a> Default for &'a UCStr<u16> {
    #[inline]
    fn default() -> Self {
        const SLICE: &[u16] = &[UChar::NUL];
        unsafe { UCStr::from_slice_unchecked(SLICE) }
    }
}

impl<'a> Default for &'a UCStr<u32> {
    #[inline]
    fn default() -> Self {
        const SLICE: &[u32] = &[UChar::NUL];
        unsafe { UCStr::from_slice_unchecked(SLICE) }
    }
}

#[cfg(feature = "alloc")]
impl<C: UChar> Default for Box<UCStr<C>> {
    #[inline]
    fn default() -> Box<UCStr<C>> {
        let boxed: Box<[C]> = Box::from([UChar::NUL]);
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut UCStr<C>) }
    }
}

#[cfg(feature = "alloc")]
impl<'a, C: UChar> From<&'a UCStr<C>> for Box<UCStr<C>> {
    #[inline]
    fn from(s: &'a UCStr<C>) -> Box<UCStr<C>> {
        let boxed: Box<[C]> = Box::from(s.as_slice_with_nul());
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut UCStr<C>) }
    }
}

impl<C: UChar> PartialEq<crate::UStr<C>> for UCStr<C> {
    #[inline]
    fn eq(&self, other: &crate::UStr<C>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<C: UChar> PartialOrd<crate::UStr<C>> for UCStr<C> {
    #[inline]
    fn partial_cmp(&self, other: &crate::UStr<C>) -> Option<core::cmp::Ordering> {
        self.as_ustr().partial_cmp(other)
    }
}

/// C-style string reference for [`U16CString`][crate::U16CString].
///
/// [`U16CStr`] is aware of nul values. Unless unchecked conversions are used, all [`U16CStr`]
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-16 data. These strings are intended to
/// be used with FFI functions such as Windows API that may require nul-terminated strings.
///
/// [`U16CStr`] can be converted to and from many other string types, including
/// [`U16String`][crate::U16String], [`OsString`][std::ffi::OsString], and [`String`], making proper
/// Unicode FFI safe and easy.
pub type U16CStr = UCStr<u16>;

/// C-style string reference for [`U32CString`][crate::U32CString].
///
/// [`U32CStr`] is aware of nul values. Unless unchecked conversions are used, all [`U32CStr`]
/// strings end with a nul-terminator in the underlying buffer and contain no internal nul values.
/// The strings may still contain invalid or ill-formed UTF-32 data. These strings are intended to
/// be used with FFI functions such as Windows API that may require nul-terminated strings.
///
/// [`U32CStr`] can be converted to and from many other string types, including
/// [`U32String`][crate::U32String], [`OsString`][std::ffi::OsString], and [`String`], making proper
/// Unicode FFI safe and easy.
pub type U32CStr = UCStr<u32>;

/// Alias for [`U16CStr`] or [`U32CStr`] depending on platform. Intended to match typical C
/// `wchar_t` size on platform.
pub type WideCStr = UCStr<WideChar>;

/// Helper struct for printing [`UCStr`] values with [`format!`] and `{}`.
///
/// A [`UCStr`] might contain ill-formed UTF encoding. This struct implements the
/// [`Display`][std::fmt::Display] trait in a way that decoding the string is lossy but no heap
/// allocations are performed, such as by [`to_string_lossy`][UCStr::to_string_lossy]. It is created
/// by the [`display`][UCStr::display] method on [`UCStr`].
///
/// By default, invalid Unicode data is replaced with
/// [`U+FFFD REPLACEMENT CHARACTER`][std::char::REPLACEMENT_CHARACTER] (�). If you wish to simply
/// skip any invalid Uncode data and forego the replacement, you may use the
/// [alternate formatting][std::fmt#sign0] with `{:#}`.
pub struct Display<'a, C: UChar> {
    str: &'a UCStr<C>,
}

impl<'a> core::fmt::Debug for Display<'a, u16> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.str, f)
    }
}

impl<'a> core::fmt::Debug for Display<'a, u32> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.str, f)
    }
}

impl<'a> core::fmt::Display for Display<'a, u16> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for c in crate::decode_utf16_lossy(self.str.as_slice().iter().copied()) {
            // Allow alternate {:#} format which skips replacment chars entirely
            if c != core::char::REPLACEMENT_CHARACTER || !f.alternate() {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

impl<'a> core::fmt::Display for Display<'a, u32> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for c in crate::decode_utf32_lossy(self.str.as_slice().iter().copied()) {
            // Allow alternate {:#} format which skips replacment chars entirely
            if c != core::char::REPLACEMENT_CHARACTER || !f.alternate() {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}
