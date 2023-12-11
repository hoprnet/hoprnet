//! Metrics and descriptions for the given request-response cycle.
//!
//! # Examples
//!
//! ```
//! # fn main() -> http_types::Result<()> {
//! #
//! use http_types::Response;
//! use http_types::trace::{ServerTiming, Metric};
//!
//! let mut timings = ServerTiming::new();
//! timings.push(Metric::new("server".to_owned(), None, None)?);
//!
//! let mut res = Response::new(200);
//! timings.apply(&mut res);
//!
//! let timings = ServerTiming::from_headers(res)?.unwrap();
//! let entry = timings.iter().next().unwrap();
//! assert_eq!(entry.name(), "server");
//! #
//! # Ok(()) }
//! ```

mod metric;
mod parse;

pub use metric::Metric;
use parse::parse_header;

use std::convert::AsMut;
use std::fmt::Write;
use std::iter::Iterator;
use std::option;
use std::slice;

use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, SERVER_TIMING};

/// Metrics and descriptions for the given request-response cycle.
///
/// # Specifications
///
/// - [Server Timing (Working Draft)](https://w3c.github.io/server-timing/#the-server-timing-header-field)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::trace::{ServerTiming, Metric};
///
/// let mut timings = ServerTiming::new();
/// timings.push(Metric::new("server".to_owned(), None, None)?);
///
/// let mut res = Response::new(200);
/// timings.apply(&mut res);
///
/// let timings = ServerTiming::from_headers(res)?.unwrap();
/// let entry = timings.iter().next().unwrap();
/// assert_eq!(entry.name(), "server");
/// #
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct ServerTiming {
    timings: Vec<Metric>,
}

impl ServerTiming {
    /// Create a new instance of `ServerTiming`.
    pub fn new() -> Self {
        Self { timings: vec![] }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut timings = vec![];
        let headers = match headers.as_ref().get(SERVER_TIMING) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        for value in headers {
            parse_header(value.as_str(), &mut timings)?;
        }
        Ok(Some(Self { timings }))
    }

    /// Sets the `Server-Timing` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(SERVER_TIMING, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        SERVER_TIMING
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, timing) in self.timings.iter().enumerate() {
            let timing: HeaderValue = timing.clone().into();
            match n {
                0 => write!(output, "{}", timing).unwrap(),
                _ => write!(output, ", {}", timing).unwrap(),
            };
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Push an entry into the list of entries.
    pub fn push(&mut self, entry: Metric) {
        self.timings.push(entry);
    }

    /// An iterator visiting all server timings.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.timings.iter(),
        }
    }

    /// An iterator visiting all server timings.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.timings.iter_mut(),
        }
    }
}

impl IntoIterator for ServerTiming {
    type Item = Metric;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.timings.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a ServerTiming {
    type Item = &'a Metric;
    type IntoIter = Iter<'a>;

    // #[inline]serv
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut ServerTiming {
    type Item = &'a mut Metric;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `ServerTiming`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<Metric>,
}

impl Iterator for IntoIter {
    type Item = Metric;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `ServerTiming`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, Metric>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Metric;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `ServerTiming`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, Metric>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Metric;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for ServerTiming {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut timings = ServerTiming::new();
        timings.push(Metric::new("server".to_owned(), None, None)?);

        let mut headers = Headers::new();
        timings.apply(&mut headers);

        let timings = ServerTiming::from_headers(headers)?.unwrap();
        let entry = timings.iter().next().unwrap();
        assert_eq!(entry.name(), "server");
        Ok(())
    }

    #[test]
    fn to_header_values() -> crate::Result<()> {
        let mut timings = ServerTiming::new();
        timings.push(Metric::new("server".to_owned(), None, None)?);

        let mut headers = Headers::new();
        timings.apply(&mut headers);

        let timings = ServerTiming::from_headers(headers)?.unwrap();
        let entry = timings.iter().next().unwrap();
        assert_eq!(entry.name(), "server");
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(SERVER_TIMING, "server; <nori ate your param omnom>");
        let err = ServerTiming::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
