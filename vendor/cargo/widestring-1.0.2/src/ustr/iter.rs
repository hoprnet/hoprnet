use crate::{
    error::{DecodeUtf16Error, DecodeUtf32Error},
    iter::{DecodeUtf16, DecodeUtf16Lossy, DecodeUtf32, DecodeUtf32Lossy},
};
use core::{
    iter::{Copied, DoubleEndedIterator, ExactSizeIterator, FusedIterator},
    slice::Iter,
};

/// An iterator over UTF-16 decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the `chars` method on strings. See its documentation for more.
#[derive(Clone)]
pub struct CharsUtf16<'a> {
    inner: DecodeUtf16<Copied<Iter<'a, u16>>>,
}

impl<'a> CharsUtf16<'a> {
    pub(crate) fn new(s: &'a [u16]) -> Self {
        Self {
            inner: crate::decode_utf16(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsUtf16<'a> {
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

impl<'a> FusedIterator for CharsUtf16<'a> {}

impl<'a> DoubleEndedIterator for CharsUtf16<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a> core::fmt::Debug for CharsUtf16<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_utf16_iter(self.clone(), f)
    }
}

/// An iterator over UTF-32 decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the `chars` method on strings. See its documentation for more.
#[derive(Clone)]
pub struct CharsUtf32<'a> {
    inner: DecodeUtf32<Copied<Iter<'a, u32>>>,
}

impl<'a> CharsUtf32<'a> {
    pub(crate) fn new(s: &'a [u32]) -> Self {
        Self {
            inner: crate::decode_utf32(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsUtf32<'a> {
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

impl<'a> FusedIterator for CharsUtf32<'a> {}

impl<'a> DoubleEndedIterator for CharsUtf32<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a> ExactSizeIterator for CharsUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> core::fmt::Debug for CharsUtf32<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_utf32_iter(self.clone(), f)
    }
}

/// A lossy iterator over UTF-16 decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the `chars_lossy` method on strings. See its documentation for more.
#[derive(Clone)]
pub struct CharsLossyUtf16<'a> {
    iter: DecodeUtf16Lossy<Copied<Iter<'a, u16>>>,
}

impl<'a> CharsLossyUtf16<'a> {
    pub(crate) fn new(s: &'a [u16]) -> Self {
        Self {
            iter: crate::decode_utf16_lossy(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsLossyUtf16<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharsLossyUtf16<'a> {}

impl<'a> DoubleEndedIterator for CharsLossyUtf16<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a> core::fmt::Debug for CharsLossyUtf16<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_char_iter(self.clone(), f)
    }
}

/// A lossy iterator over UTF-32 decoded [`char`][prim@char]s of a string slice.
///
/// This struct is created by the `chars_lossy` method on strings. See its documentation for more.
#[derive(Clone)]
pub struct CharsLossyUtf32<'a> {
    iter: DecodeUtf32Lossy<Copied<Iter<'a, u32>>>,
}

impl<'a> CharsLossyUtf32<'a> {
    pub(crate) fn new(s: &'a [u32]) -> Self {
        Self {
            iter: crate::decode_utf32_lossy(s.iter().copied()),
        }
    }
}

impl<'a> Iterator for CharsLossyUtf32<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharsLossyUtf32<'a> {}

impl<'a> DoubleEndedIterator for CharsLossyUtf32<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for CharsLossyUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> core::fmt::Debug for CharsLossyUtf32<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::debug_fmt_char_iter(self.clone(), f)
    }
}

/// An iterator over the decoded [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the `char_indices` method on strings. See its documentation for
/// more.
#[derive(Debug, Clone)]
pub struct CharIndicesUtf16<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsUtf16<'a>,
}

impl<'a> CharIndicesUtf16<'a> {
    pub(crate) fn new(s: &'a [u16]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsUtf16::new(s),
        }
    }

    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> Iterator for CharIndicesUtf16<'a> {
    type Item = (usize, Result<char, DecodeUtf16Error>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(c)) => {
                let idx = self.forward_offset;
                self.forward_offset += c.len_utf16();
                Some((idx, Ok(c)))
            }
            Some(Err(e)) => {
                let idx = self.forward_offset;
                self.forward_offset += 1;
                Some((idx, Err(e)))
            }
            None => None,
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
        match self.iter.next_back() {
            Some(Ok(c)) => {
                self.back_offset -= c.len_utf16();
                Some((self.back_offset, Ok(c)))
            }
            Some(Err(e)) => {
                self.back_offset -= 1;
                Some((self.back_offset, Err(e)))
            }
            None => None,
        }
    }
}

/// An iterator over the decoded [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the `char_indices` method on strings. See its documentation for
/// more.
#[derive(Debug, Clone)]
pub struct CharIndicesUtf32<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsUtf32<'a>,
}

impl<'a> CharIndicesUtf32<'a> {
    pub(crate) fn new(s: &'a [u32]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsUtf32::new(s),
        }
    }

    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> Iterator for CharIndicesUtf32<'a> {
    type Item = (usize, Result<char, DecodeUtf32Error>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(c)) => {
                let idx = self.forward_offset;
                self.forward_offset += 1;
                Some((idx, Ok(c)))
            }
            Some(Err(e)) => {
                let idx = self.forward_offset;
                self.forward_offset += 1;
                Some((idx, Err(e)))
            }
            None => None,
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
        match self.iter.next_back() {
            Some(Ok(c)) => {
                self.back_offset -= 1;
                Some((self.back_offset, Ok(c)))
            }
            Some(Err(e)) => {
                self.back_offset -= 1;
                Some((self.back_offset, Err(e)))
            }
            None => None,
        }
    }
}

impl<'a> ExactSizeIterator for CharIndicesUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A lossy iterator over the [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the `char_indices_lossy` method on strings. See its documentation
/// for more.
#[derive(Debug, Clone)]
pub struct CharIndicesLossyUtf16<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsLossyUtf16<'a>,
}

impl<'a> CharIndicesLossyUtf16<'a> {
    pub(crate) fn new(s: &'a [u16]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsLossyUtf16::new(s),
        }
    }

    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> Iterator for CharIndicesLossyUtf16<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(c) => {
                let idx = self.forward_offset;
                self.forward_offset += c.len_utf16();
                Some((idx, c))
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharIndicesLossyUtf16<'a> {}

impl<'a> DoubleEndedIterator for CharIndicesLossyUtf16<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            Some(c) => {
                self.back_offset -= c.len_utf16();
                Some((self.back_offset, c))
            }
            None => None,
        }
    }
}

/// A lossy iterator over the [`char`][prim@char]s of a string slice, and their positions.
///
/// This struct is created by the `char_indices_lossy` method on strings. See its documentation
/// for more.
#[derive(Debug, Clone)]
pub struct CharIndicesLossyUtf32<'a> {
    forward_offset: usize,
    back_offset: usize,
    iter: CharsLossyUtf32<'a>,
}

impl<'a> CharIndicesLossyUtf32<'a> {
    pub(crate) fn new(s: &'a [u32]) -> Self {
        Self {
            forward_offset: 0,
            back_offset: s.len(),
            iter: CharsLossyUtf32::new(s),
        }
    }

    /// Returns the position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.forward_offset
    }
}

impl<'a> Iterator for CharIndicesLossyUtf32<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(c) => {
                let idx = self.forward_offset;
                self.forward_offset += 1;
                Some((idx, c))
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> FusedIterator for CharIndicesLossyUtf32<'a> {}

impl<'a> DoubleEndedIterator for CharIndicesLossyUtf32<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            Some(c) => {
                self.back_offset -= 1;
                Some((self.back_offset, c))
            }
            None => None,
        }
    }
}

impl<'a> ExactSizeIterator for CharIndicesLossyUtf32<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
