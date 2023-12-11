//! Apply the HTTP method if the ETags do not match.
//!
//! This is used to update caches or to prevent uploading a new resource when
//! one already exists.

use crate::conditional::ETag;
use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, IF_NONE_MATCH};

use std::fmt::{self, Debug, Write};
use std::iter::Iterator;
use std::option;
use std::slice;

/// Apply the HTTP method if the ETags do not match.
///
/// This is used to update caches or to prevent uploading a new resource when
/// one already exists.
///
/// # Specifications
///
/// - [RFC 7232, section 3.2: If-None-Match](https://tools.ietf.org/html/rfc7232#section-3.2)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::conditional::{IfNoneMatch, ETag};
///
/// let mut entries = IfNoneMatch::new();
/// entries.push(ETag::new("0xcafebeef".to_string()));
/// entries.push(ETag::new("0xbeefcafe".to_string()));
///
/// let mut res = Response::new(200);
/// entries.apply(&mut res);
///
/// let entries = IfNoneMatch::from_headers(res)?.unwrap();
/// let mut entries = entries.iter();
/// assert_eq!(entries.next().unwrap(), &ETag::new("0xcafebeef".to_string()));
/// assert_eq!(entries.next().unwrap(), &ETag::new("0xbeefcafe".to_string()));
/// #
/// # Ok(()) }
/// ```
pub struct IfNoneMatch {
    entries: Vec<ETag>,
    wildcard: bool,
}

impl IfNoneMatch {
    /// Create a new instance of `IfNoneMatch`.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            wildcard: false,
        }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let headers = match headers.as_ref().get(IF_NONE_MATCH) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut wildcard = false;
        for value in headers {
            for part in value.as_str().trim().split(',') {
                let part = part.trim();
                if part == "*" {
                    wildcard = true;
                    continue;
                }
                entries.push(ETag::from_str(part)?);
            }
        }

        Ok(Some(Self { entries, wildcard }))
    }

    /// Sets the `If-None-Match` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(IF_NONE_MATCH, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        IF_NONE_MATCH
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, etag) in self.entries.iter().enumerate() {
            match n {
                0 => write!(output, "{}", etag.to_string()).unwrap(),
                _ => write!(output, ", {}", etag.to_string()).unwrap(),
            };
        }

        if self.wildcard {
            match output.len() {
                0 => write!(output, "*").unwrap(),
                _ => write!(output, ", *").unwrap(),
            };
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Push a directive into the list of entries.
    pub fn push(&mut self, directive: impl Into<ETag>) {
        self.entries.push(directive.into());
    }

    /// Returns `true` if a wildcard directive was set.
    pub fn wildcard(&self) -> bool {
        self.wildcard
    }

    /// Set the wildcard directive.
    pub fn set_wildcard(&mut self, wildcard: bool) {
        self.wildcard = wildcard
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

impl IntoIterator for IfNoneMatch {
    type Item = ETag;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a IfNoneMatch {
    type Item = &'a ETag;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut IfNoneMatch {
    type Item = &'a mut ETag;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `IfNoneMatch`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<ETag>,
}

impl Iterator for IntoIter {
    type Item = ETag;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `IfNoneMatch`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, ETag>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a ETag;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `IfNoneMatch`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, ETag>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut ETag;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for IfNoneMatch {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for IfNoneMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for directive in &self.entries {
            list.entry(directive);
        }
        list.finish()
    }
}

#[cfg(test)]
mod test {
    use crate::conditional::{ETag, IfNoneMatch};
    use crate::Response;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut entries = IfNoneMatch::new();
        entries.push(ETag::new("0xcafebeef".to_string()));
        entries.push(ETag::new("0xbeefcafe".to_string()));

        let mut res = Response::new(200);
        entries.apply(&mut res);

        let entries = IfNoneMatch::from_headers(res)?.unwrap();
        let mut entries = entries.iter();
        assert_eq!(
            entries.next().unwrap(),
            &ETag::new("0xcafebeef".to_string())
        );
        assert_eq!(
            entries.next().unwrap(),
            &ETag::new("0xbeefcafe".to_string())
        );
        Ok(())
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut entries = IfNoneMatch::new();
        entries.push(ETag::new("0xcafebeef".to_string()));
        entries.set_wildcard(true);

        let mut res = Response::new(200);
        entries.apply(&mut res);

        let entries = IfNoneMatch::from_headers(res)?.unwrap();
        assert!(entries.wildcard());
        let mut entries = entries.iter();
        assert_eq!(
            entries.next().unwrap(),
            &ETag::new("0xcafebeef".to_string())
        );
        Ok(())
    }
}
