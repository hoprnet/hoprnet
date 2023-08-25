use crate::{
    debug_fmt_char_iter, decode_utf16, decode_utf32,
    iter::{DecodeUtf16, DecodeUtf32},
};
use core::{
    fmt::Write,
    iter::{Copied, DoubleEndedIterator, ExactSizeIterator, FlatMap, FusedIterator},
    slice::Iter,
};

/// An iterator over the [`char`]s of a UTF-16 string slice
///
/// This struct is created by the [`chars`][crate::Utf16Str::chars] method on
/// [`Utf16Str`][crate::Utf16Str]. See its documentation for more.
#[derive(Clone)]
pub struct CharsUtf16<'a> {
    iter: DecodeUtf16<Copied<Iter<'a, u16>>>,
}

impl<'a> CharsUtf16<'a> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: decode_utf16(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsUtf16<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Utf16Str already ensures valid surrogate pairs
        self.iter.next().map(|r| r.unwrap())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharsUtf16<'a> {}

impl<'a> DoubleEndedIterator for CharsUtf16<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|r| r.unwrap())
    }
}

impl<'a> core::fmt::Debug for CharsUtf16<'a> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        debug_fmt_char_iter(self.clone(), f)
    }
}

impl<'a> core::fmt::Display for CharsUtf16<'a> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.clone().try_for_each(|c| f.write_char(c))
    }
}

/// An iterator over the [`char`]s of a UTF-32 string slice
///
/// This struct is created by the [`chars`][crate::Utf32Str::chars] method on
/// [`Utf32Str`][crate::Utf32Str]. See its documentation for more.
#[derive(Clone)]
pub struct CharsUtf32<'a> {
    iter: DecodeUtf32<Copied<Iter<'a, u32>>>,
}

impl<'a> CharsUtf32<'a> {
    pub(super) fn new(s: &'a [u32]) -> Self {
        Self {
            iter: decode_utf32(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsUtf32<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Utf32Str already ensures valid code points
        self.iter.next().map(|r| r.unwrap())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for CharsUtf32<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        // Utf32Str already ensures valid code points
        self.iter.next_back().map(|r| r.unwrap())
    }
}

impl<'a> FusedIterator for CharsUtf32<'a> {}

impl<'a> ExactSizeIterator for CharsUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> core::fmt::Debug for CharsUtf32<'a> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        debug_fmt_char_iter(self.clone(), f)
    }
}

impl<'a> core::fmt::Display for CharsUtf32<'a> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.clone().try_for_each(|c| f.write_char(c))
    }
}

/// An iterator over the [`char`]s of a string slice, and their positions
///
/// This struct is created by the [`char_indices`][crate::Utf16Str::char_indices] method on
/// [`Utf16Str`][crate::Utf16Str]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct CharIndicesUtf16<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsUtf16<'a>,
}

impl<'a> CharIndicesUtf16<'a> {
    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> CharIndicesUtf16<'a> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsUtf16::new(s),
        }
    }
}

impl<'a> Iterator for CharIndicesUtf16<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.iter.next();
        if let Some(c) = result {
            let offset = self.forward_offset;
            self.forward_offset += c.len_utf16();
            Some((offset, c))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharIndicesUtf16<'a> {}

impl<'a> DoubleEndedIterator for CharIndicesUtf16<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let result = self.iter.next_back();
        if let Some(c) = result {
            self.back_offset -= c.len_utf16();
            Some((self.back_offset, c))
        } else {
            None
        }
    }
}

/// An iterator over the [`char`]s of a string slice, and their positions
///
/// This struct is created by the [`char_indices`][crate::Utf32Str::char_indices] method on
/// [`Utf32Str`][crate::Utf32Str]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct CharIndicesUtf32<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsUtf32<'a>,
}

impl<'a> CharIndicesUtf32<'a> {
    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> CharIndicesUtf32<'a> {
    pub(super) fn new(s: &'a [u32]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsUtf32::new(s),
        }
    }
}

impl<'a> Iterator for CharIndicesUtf32<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.iter.next();
        if let Some(c) = result {
            let offset = self.forward_offset;
            self.forward_offset += 1;
            Some((offset, c))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharIndicesUtf32<'a> {}

impl<'a> DoubleEndedIterator for CharIndicesUtf32<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let result = self.iter.next_back();
        if let Some(c) = result {
            self.back_offset -= 1;
            Some((self.back_offset, c))
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for CharIndicesUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// The return type of [`Utf16Str::escape_debug`][crate::Utf16Str::escape_debug].
#[derive(Debug, Clone)]
pub struct EscapeDebug<I> {
    iter: FlatMap<I, core::char::EscapeDebug, fn(char) -> core::char::EscapeDebug>,
}

impl<'a> EscapeDebug<CharsUtf16<'a>> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: CharsUtf16::new(s).flat_map(|c| c.escape_debug()),
        }
    }
}

impl<'a> EscapeDebug<CharsUtf32<'a>> {
    pub(super) fn new(s: &'a [u32]) -> Self {
        Self {
            iter: CharsUtf32::new(s).flat_map(|c| c.escape_debug()),
        }
    }
}

/// The return type of [`Utf16Str::escape_default`][crate::Utf16Str::escape_default].
#[derive(Debug, Clone)]
pub struct EscapeDefault<I> {
    iter: FlatMap<I, core::char::EscapeDefault, fn(char) -> core::char::EscapeDefault>,
}

impl<'a> EscapeDefault<CharsUtf16<'a>> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: CharsUtf16::new(s).flat_map(|c| c.escape_default()),
        }
    }
}

impl<'a> EscapeDefault<CharsUtf32<'a>> {
    pub(super) fn new(s: &'a [u32]) -> Self {
        Self {
            iter: CharsUtf32::new(s).flat_map(|c| c.escape_default()),
        }
    }
}

/// The return type of [`Utf16Str::escape_unicode`][crate::Utf16Str::escape_unicode].
#[derive(Debug, Clone)]
pub struct EscapeUnicode<I> {
    iter: FlatMap<I, core::char::EscapeUnicode, fn(char) -> core::char::EscapeUnicode>,
}

impl<'a> EscapeUnicode<CharsUtf16<'a>> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: CharsUtf16::new(s).flat_map(|c| c.escape_unicode()),
        }
    }
}

impl<'a> EscapeUnicode<CharsUtf32<'a>> {
    pub(super) fn new(s: &'a [u32]) -> Self {
        Self {
            iter: CharsUtf32::new(s).flat_map(|c| c.escape_unicode()),
        }
    }
}

macro_rules! escape_impls {
    ($($name:ident),+) => {$(
        impl<I> core::fmt::Display for $name<I> where I: Iterator<Item = char> + Clone {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.clone().try_for_each(|c| f.write_char(c))
            }
        }

        impl< I> Iterator for $name<I> where I: Iterator<Item = char> {
            type Item = char;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let (lower, upper) = self.iter.size_hint();
                // Worst case, every char has to be unicode escaped as \u{NNNNNN}
                (lower, upper.and_then(|len| len.checked_mul(10)))
            }
        }

        impl<I> FusedIterator for $name<I> where I: Iterator<Item = char> + FusedIterator {}
    )+}
}

escape_impls!(EscapeDebug, EscapeDefault, EscapeUnicode);

/// An iterator over the [`u16`] code units of a UTF-16 string slice
///
/// This struct is created by the [`code_units`][crate::Utf16Str::code_units] method on
/// [`Utf16Str`][crate::Utf16Str]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct CodeUnits<'a> {
    iter: Copied<Iter<'a, u16>>,
}

impl<'a> CodeUnits<'a> {
    pub(super) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: s.iter().copied(),
        }
    }
}

impl<'a> Iterator for CodeUnits<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CodeUnits<'a> {}

impl<'a> DoubleEndedIterator for CodeUnits<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for CodeUnits<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
