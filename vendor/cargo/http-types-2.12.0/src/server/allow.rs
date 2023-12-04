//! List the set of methods supported by a resource.

use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, ALLOW};
use crate::Method;

use std::collections::{hash_set, HashSet};
use std::fmt::{self, Debug, Write};
use std::iter::Iterator;
use std::option;
use std::str::FromStr;

/// List the set of methods supported by a resource.
///
/// # Specifications
///
/// - [RFC 7231, section 7.4.1: Allow](https://tools.ietf.org/html/rfc7231#section-7.4.1)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::{Method, Response};
/// use http_types::server::Allow;
///
/// let mut allow = Allow::new();
/// allow.insert(Method::Put);
/// allow.insert(Method::Post);
///
/// let mut res = Response::new(200);
/// allow.apply(&mut res);
///
/// let allow = Allow::from_headers(res)?.unwrap();
/// assert!(allow.contains(Method::Put));
/// assert!(allow.contains(Method::Post));
/// #
/// # Ok(()) }
/// ```
pub struct Allow {
    entries: HashSet<Method>,
}

impl Allow {
    /// Create a new instance of `Allow`.
    pub fn new() -> Self {
        Self {
            entries: HashSet::new(),
        }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = HashSet::new();
        let headers = match headers.as_ref().get(ALLOW) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        for value in headers {
            for part in value.as_str().trim().split(',') {
                let method = Method::from_str(part.trim())?;
                entries.insert(method);
            }
        }

        Ok(Some(Self { entries }))
    }

    /// Sets the `Allow` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(ALLOW, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        ALLOW
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, method) in self.entries.iter().enumerate() {
            match n {
                0 => write!(output, "{}", method).unwrap(),
                _ => write!(output, ", {}", method).unwrap(),
            };
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Push a method into the set of methods.
    pub fn insert(&mut self, method: Method) {
        self.entries.insert(method);
    }

    /// An iterator visiting all server entries.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// Returns `true` if the header contains the `Method`.
    pub fn contains(&self, method: Method) -> bool {
        self.entries.contains(&method)
    }
}

impl IntoIterator for Allow {
    type Item = Method;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Allow {
    type Item = &'a Method;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A borrowing iterator over entries in `Allow`.
#[derive(Debug)]
pub struct IntoIter {
    inner: hash_set::IntoIter<Method>,
}

impl Iterator for IntoIter {
    type Item = Method;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `Allow`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: hash_set::Iter<'a, Method>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Method;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for Allow {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for Allow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for method in &self.entries {
            list.entry(method);
        }
        list.finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut allow = Allow::new();
        allow.insert(Method::Put);
        allow.insert(Method::Post);

        let mut headers = Headers::new();
        allow.apply(&mut headers);

        let allow = Allow::from_headers(headers)?.unwrap();
        assert!(allow.contains(Method::Put));
        assert!(allow.contains(Method::Post));
        Ok(())
    }
}
