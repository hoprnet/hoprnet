use crate::cache::CacheDirective;
use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, CACHE_CONTROL};

use std::fmt::{self, Debug, Write};
use std::iter::Iterator;
use std::option;
use std::slice;

/// A Cache-Control header.
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::cache::{CacheControl, CacheDirective};
/// let mut entries = CacheControl::new();
/// entries.push(CacheDirective::Immutable);
/// entries.push(CacheDirective::NoStore);
///
/// let mut res = Response::new(200);
/// entries.apply(&mut res);
///
/// let entries = CacheControl::from_headers(res)?.unwrap();
/// let mut entries = entries.iter();
/// assert_eq!(entries.next().unwrap(), &CacheDirective::Immutable);
/// assert_eq!(entries.next().unwrap(), &CacheDirective::NoStore);
/// #
/// # Ok(()) }
/// ```
pub struct CacheControl {
    entries: Vec<CacheDirective>,
}

impl CacheControl {
    /// Create a new instance of `CacheControl`.
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let headers = match headers.as_ref().get(CACHE_CONTROL) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        for value in headers {
            for part in value.as_str().trim().split(',') {
                // Try and parse a directive from a str. If the directive is
                // unkown we skip it.
                if let Some(entry) = CacheDirective::from_str(part)? {
                    entries.push(entry);
                }
            }
        }

        Ok(Some(Self { entries }))
    }

    /// Sets the `Server-Timing` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(CACHE_CONTROL, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        CACHE_CONTROL
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, directive) in self.entries.iter().enumerate() {
            let directive: HeaderValue = directive.clone().into();
            match n {
                0 => write!(output, "{}", directive).unwrap(),
                _ => write!(output, ", {}", directive).unwrap(),
            };
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }
    /// Push a directive into the list of entries.
    pub fn push(&mut self, directive: CacheDirective) {
        self.entries.push(directive);
    }

    /// An iterator visiting all server entries.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// An iterator visiting all server entries.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.entries.iter_mut(),
        }
    }
}

impl IntoIterator for CacheControl {
    type Item = CacheDirective;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a CacheControl {
    type Item = &'a CacheDirective;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut CacheControl {
    type Item = &'a mut CacheDirective;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `CacheControl`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<CacheDirective>,
}

impl Iterator for IntoIter {
    type Item = CacheDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `CacheControl`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, CacheDirective>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a CacheDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `CacheControl`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, CacheDirective>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut CacheDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for CacheControl {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for directive in &self.entries {
            list.entry(directive);
        }
        list.finish()
    }
}
