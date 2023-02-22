//! Iterators for working with wide strings.

use crate::{error::DecodeUtf32Error, U16CStr, U16Str, U32CStr, U32Str};
use core::{
    char::{self, DecodeUtf16Error},
    fmt::Write,
    iter::{Copied, FusedIterator},
    slice::Iter,
};

#[doc(no_inline)]
pub use core::char::DecodeUtf16;

/// An iterator that lossily decodes possibly ill-formed UTF-16 encoded code points from an iterator
/// of `u16`s.
///
/// Any unpaired UTF-16 surrogate values are replaced by
/// [`U+FFFD REPLACEMENT_CHARACTER`][core::char::REPLACEMENT_CHARACTER] (�).
#[derive(Debug, Clone)]
pub struct DecodeUtf16Lossy<I>
where
    I: Iterator<Item = u16>,
{
    pub(crate) iter: DecodeUtf16<I>,
}

impl<I> Iterator for DecodeUtf16Lossy<I>
where
    I: Iterator<Item = u16>,
{
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|res| res.unwrap_or(core::char::REPLACEMENT_CHARACTER))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for DecodeUtf16Lossy<I> where I: Iterator<Item = u16> + FusedIterator {}

/// An iterator that decodes UTF-32 encoded code points from an iterator of `u32`s.
#[derive(Debug, Clone)]
pub struct DecodeUtf32<I>
where
    I: Iterator<Item = u32>,
{
    pub(crate) iter: I,
}

impl<I> Iterator for DecodeUtf32<I>
where
    I: Iterator<Item = u32>,
{
    type Item = Result<char, DecodeUtf32Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|u| core::char::from_u32(u).ok_or_else(|| DecodeUtf32Error::new(u)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for DecodeUtf32<I> where I: Iterator<Item = u32> + FusedIterator {}

/// An iterator that lossily decodes possibly ill-formed UTF-32 encoded code points from an iterator
/// of `u32`s.
///
/// Any invalid UTF-32 values are replaced by
/// [`U+FFFD REPLACEMENT_CHARACTER`][core::char::REPLACEMENT_CHARACTER] (�).
#[derive(Debug, Clone)]
pub struct DecodeUtf32Lossy<I>
where
    I: Iterator<Item = u32>,
{
    pub(crate) iter: DecodeUtf32<I>,
}

impl<I> Iterator for DecodeUtf32Lossy<I>
where
    I: Iterator<Item = u32>,
{
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|res| res.unwrap_or(core::char::REPLACEMENT_CHARACTER))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for DecodeUtf32Lossy<I> where I: Iterator<Item = u32> + FusedIterator {}

/// An iterator over decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the [`chars`][crate::UStr<u16>::chars] method on [`U16Str`] and [`U16CStr`].
/// See its documentation for more.
#[derive(Clone)]
pub struct Utf16Chars<'a> {
    inner: DecodeUtf16<Copied<Iter<'a, u16>>>,
}

impl<'a> Utf16Chars<'a> {
    pub(super) fn from_ustr(s: &'a U16Str) -> Self {
        Self {
            inner: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }

    pub(super) fn from_ucstr(s: &'a U16CStr) -> Self {
        Self {
            inner: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }
}

impl<'a> Iterator for Utf16Chars<'a> {
    type Item = Result<char, DecodeUtf16Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> FusedIterator for Utf16Chars<'a> {}

impl<'a> core::fmt::Debug for Utf16Chars<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_utf16_iter(self.clone(), f)
    }
}

/// An iterator over decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the [`chars`][crate::UStr<u32>::chars] method on [`U32Str`] and [`U32CStr`].
/// See its documentation for more.
#[derive(Clone)]
pub struct Utf32Chars<'a> {
    inner: DecodeUtf32<Copied<Iter<'a, u32>>>,
}

impl<'a> Utf32Chars<'a> {
    pub(super) fn from_ustr(s: &'a U32Str) -> Self {
        Self {
            inner: crate::decode_utf32(s.as_slice().iter().copied()),
        }
    }

    pub(super) fn from_ucstr(s: &'a U32CStr) -> Self {
        Self {
            inner: crate::decode_utf32(s.as_slice().iter().copied()),
        }
    }
}

impl<'a> Iterator for Utf32Chars<'a> {
    type Item = Result<char, DecodeUtf32Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> FusedIterator for Utf32Chars<'a> {}

impl<'a> core::fmt::Debug for Utf32Chars<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_utf32_iter(self.clone(), f)
    }
}

/// A lossy iterator over decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the [`chars`][crate::UStr<u16>::chars] method on [`UStr`][crate::UStr] and
/// [`UCStr`][crate::UCStr]. See its documentation for more.
#[derive(Clone)]
pub struct CharsLossy<'a> {
    inner: CharsDecoderLossy<'a>,
}

#[derive(Clone)]
enum CharsDecoderLossy<'a> {
    Utf16(DecodeUtf16Lossy<Copied<Iter<'a, u16>>>),
    Utf32(DecodeUtf32Lossy<Copied<Iter<'a, u32>>>),
}

impl<'a> CharsLossy<'a> {
    pub(super) fn from_u16str(s: &'a U16Str) -> Self {
        Self {
            inner: CharsDecoderLossy::Utf16(crate::decode_utf16_lossy(
                s.as_slice().iter().copied(),
            )),
        }
    }

    pub(super) fn from_u16cstr(s: &'a U16CStr) -> Self {
        Self {
            inner: CharsDecoderLossy::Utf16(crate::decode_utf16_lossy(
                s.as_slice().iter().copied(),
            )),
        }
    }

    pub(super) fn from_u32str(s: &'a U32Str) -> Self {
        Self {
            inner: CharsDecoderLossy::Utf32(crate::decode_utf32_lossy(
                s.as_slice().iter().copied(),
            )),
        }
    }

    pub(super) fn from_u32cstr(s: &'a U32CStr) -> Self {
        Self {
            inner: CharsDecoderLossy::Utf32(crate::decode_utf32_lossy(
                s.as_slice().iter().copied(),
            )),
        }
    }
}

impl<'a> Iterator for CharsLossy<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CharsDecoderLossy::Utf16(iter) => iter.next(),
            CharsDecoderLossy::Utf32(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            CharsDecoderLossy::Utf16(iter) => iter.size_hint(),
            CharsDecoderLossy::Utf32(iter) => iter.size_hint(),
        }
    }
}

impl<'a> FusedIterator for CharsLossy<'a> {}

impl<'a> core::fmt::Debug for CharsLossy<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('"')?;
        for c in self.clone() {
            f.write_char(c)?;
        }
        f.write_char('"')
    }
}

/// An iterator over the [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the [`char_indices`][crate::UStr<u16>::char_indices] method on [`U16Str`] and
/// [`U16CStr`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Utf16CharIndices<'a> {
    index: usize,
    iter: DecodeUtf16<Copied<Iter<'a, u16>>>,
}

impl<'a> Utf16CharIndices<'a> {
    pub(super) fn from_ustr(s: &'a U16Str) -> Self {
        Self {
            index: 0,
            iter: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }

    pub(super) fn from_ucstr(s: &'a U16CStr) -> Self {
        Self {
            index: 0,
            iter: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }
}

impl<'a> Iterator for Utf16CharIndices<'a> {
    type Item = (Result<char, DecodeUtf16Error>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(c)) => {
                let idx = self.index;
                self.index += c.len_utf16();
                Some((Ok(c), idx))
            }
            Some(Err(e)) => {
                let idx = self.index;
                self.index += 1;
                Some((Err(e), idx))
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for Utf16CharIndices<'a> {}

/// A lossy iterator over the [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the [`char_indices_lossy`][crate::UStr<u16>::char_indices_lossy] method on
/// [`U16Str`] and [`U16CStr`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Utf16CharIndicesLossy<'a> {
    index: usize,
    iter: DecodeUtf16<Copied<Iter<'a, u16>>>,
}

impl<'a> Utf16CharIndicesLossy<'a> {
    pub(super) fn from_ustr(s: &'a U16Str) -> Self {
        Self {
            index: 0,
            iter: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }

    pub(super) fn from_ucstr(s: &'a U16CStr) -> Self {
        Self {
            index: 0,
            iter: char::decode_utf16(s.as_slice().iter().copied()),
        }
    }
}

impl<'a> Iterator for Utf16CharIndicesLossy<'a> {
    type Item = (char, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(c)) => {
                let idx = self.index;
                self.index += c.len_utf16();
                Some((c, idx))
            }
            Some(Err(_)) => {
                let idx = self.index;
                self.index += 1;
                Some((char::REPLACEMENT_CHARACTER, idx))
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for Utf16CharIndicesLossy<'a> {}
