//! Specify origins that are allowed to see values via the Resource Timing API.
//!
//! # Specifications
//!
//! - [W3C Timing-Allow-Origin header](https://w3c.github.io/resource-timing/#sec-timing-allow-origin)
//! - [WhatWG Fetch Origin header](https://fetch.spec.whatwg.org/#origin-header)
//!
//! # Examples
//!
//! ```
//! # fn main() -> http_types::Result<()> {
//! #
//! use http_types::{Response, Url};
//! use http_types::security::TimingAllowOrigin;
//!
//! let mut origins = TimingAllowOrigin::new();
//! origins.push(Url::parse("https://example.com")?);
//!
//! let mut res = Response::new(200);
//! origins.apply(&mut res);
//!
//! let origins = TimingAllowOrigin::from_headers(res)?.unwrap();
//! let origin = origins.iter().next().unwrap();
//! assert_eq!(origin, &Url::parse("https://example.com")?);
//! #
//! # Ok(()) }
//! ```

use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, TIMING_ALLOW_ORIGIN};
use crate::{Status, Url};

use std::fmt::Write;
use std::fmt::{self, Debug};
use std::iter::Iterator;
use std::option;
use std::slice;

/// Specify origins that are allowed to see values via the Resource Timing API.
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::{Response, Url};
/// use http_types::security::TimingAllowOrigin;
///
/// let mut origins = TimingAllowOrigin::new();
/// origins.push(Url::parse("https://example.com")?);
///
/// let mut res = Response::new(200);
/// origins.apply(&mut res);
///
/// let origins = TimingAllowOrigin::from_headers(res)?.unwrap();
/// let origin = origins.iter().next().unwrap();
/// assert_eq!(origin, &Url::parse("https://example.com")?);
/// #
/// # Ok(()) }
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct TimingAllowOrigin {
    origins: Vec<Url>,
    wildcard: bool,
}

impl TimingAllowOrigin {
    /// Create a new instance of `AllowOrigin`.
    pub fn new() -> Self {
        Self {
            origins: vec![],
            wildcard: false,
        }
    }

    /// Create an instance of `AllowOrigin` from a `Headers` instance.
    ///
    /// # Implementation note
    ///
    /// A header value of `"null"` is treated the same as if no header was sent.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(TIMING_ALLOW_ORIGIN) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut wildcard = false;
        let mut origins = vec![];
        for header in headers {
            for origin in header.as_str().split(',') {
                match origin.trim_start() {
                    "*" => wildcard = true,
                    r#""null""# => continue,
                    origin => {
                        let url = Url::parse(origin).status(400)?;
                        origins.push(url);
                    }
                }
            }
        }

        Ok(Some(Self { origins, wildcard }))
    }

    /// Append an origin to the list of origins.
    pub fn push(&mut self, origin: impl Into<Url>) {
        self.origins.push(origin.into());
    }

    /// Insert a `HeaderName` + `HeaderValue` pair into a `Headers` instance.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(TIMING_ALLOW_ORIGIN, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        TIMING_ALLOW_ORIGIN
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, origin) in self.origins.iter().enumerate() {
            match n {
                0 => write!(output, "{}", origin).unwrap(),
                _ => write!(output, ", {}", origin).unwrap(),
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

    /// Returns `true` if a wildcard directive was set.
    pub fn wildcard(&self) -> bool {
        self.wildcard
    }

    /// Set the wildcard directive.
    pub fn set_wildcard(&mut self, wildcard: bool) {
        self.wildcard = wildcard
    }

    /// An iterator visiting all server timings.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.origins.iter(),
        }
    }

    /// An iterator visiting all server timings.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.origins.iter_mut(),
        }
    }
}

impl IntoIterator for TimingAllowOrigin {
    type Item = Url;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.origins.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a TimingAllowOrigin {
    type Item = &'a Url;
    type IntoIter = Iter<'a>;

    // #[inline]serv
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut TimingAllowOrigin {
    type Item = &'a mut Url;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `AllowOrigin`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<Url>,
}

impl Iterator for IntoIter {
    type Item = Url;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `AllowOrigin`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, Url>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Url;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `AllowOrigin`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, Url>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Url;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// Conversion from `AllowOrigin` -> `HeaderValue`.
impl ToHeaderValues for TimingAllowOrigin {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for TimingAllowOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for origin in &self.origins {
            list.entry(origin);
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
        let mut origins = TimingAllowOrigin::new();
        origins.push(Url::parse("https://example.com")?);

        let mut headers = Headers::new();
        origins.apply(&mut headers);

        let origins = TimingAllowOrigin::from_headers(headers)?.unwrap();
        let origin = origins.iter().next().unwrap();
        assert_eq!(origin, &Url::parse("https://example.com")?);
        Ok(())
    }

    #[test]
    fn multi() -> crate::Result<()> {
        let mut origins = TimingAllowOrigin::new();
        origins.push(Url::parse("https://example.com")?);
        origins.push(Url::parse("https://mozilla.org/")?);

        let mut headers = Headers::new();
        origins.apply(&mut headers);

        let origins = TimingAllowOrigin::from_headers(headers)?.unwrap();
        let mut origins = origins.iter();
        let origin = origins.next().unwrap();
        assert_eq!(origin, &Url::parse("https://example.com")?);

        let origin = origins.next().unwrap();
        assert_eq!(origin, &Url::parse("https://mozilla.org/")?);
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(TIMING_ALLOW_ORIGIN, "server; <nori ate your param omnom>");
        let err = TimingAllowOrigin::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut origins = TimingAllowOrigin::new();
        origins.push(Url::parse("https://example.com")?);
        origins.set_wildcard(true);

        let mut headers = Headers::new();
        origins.apply(&mut headers);

        let origins = TimingAllowOrigin::from_headers(headers)?.unwrap();
        assert!(origins.wildcard());
        let origin = origins.iter().next().unwrap();
        assert_eq!(origin, &Url::parse("https://example.com")?);
        Ok(())
    }
}
