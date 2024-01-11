//! Clear browsing data (cookies, storage, cache) associated with the
//! requesting website

use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, CLEAR_SITE_DATA};

use std::fmt::{self, Debug, Write};
use std::iter::Iterator;
use std::option;
use std::slice;
use std::str::FromStr;

mod directive;

pub use directive::ClearDirective;

/// Clear browsing data (cookies, storage, cache) associated with the
/// requesting website.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Clear-Site-Data)
///
/// # Specifications
///
/// - [Clear Site Data](https://w3c.github.io/webappsec-clear-site-data/)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::cache::{ClearSiteData, ClearDirective};
///
/// let mut entries = ClearSiteData::new();
/// entries.push(ClearDirective::Cache);
/// entries.push(ClearDirective::Cookies);
///
/// let mut res = Response::new(200);
/// entries.apply(&mut res);
///
/// let entries = ClearSiteData::from_headers(res)?.unwrap();
/// let mut entries = entries.iter();
/// assert_eq!(entries.next().unwrap(), &ClearDirective::Cache);
/// assert_eq!(entries.next().unwrap(), &ClearDirective::Cookies);
/// #
/// # Ok(()) }
/// ```
pub struct ClearSiteData {
    entries: Vec<ClearDirective>,
    wildcard: bool,
}

impl ClearSiteData {
    /// Create a new instance of `ClearSiteData`.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            wildcard: false,
        }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let header_values = match headers.as_ref().get(CLEAR_SITE_DATA) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut wildcard = false;
        for value in header_values {
            for part in value.as_str().trim().split(',') {
                let part = part.trim();
                if part == r#""*""# {
                    wildcard = true;
                    continue;
                }
                entries.push(ClearDirective::from_str(part)?);
            }
        }

        Ok(Some(Self { entries, wildcard }))
    }

    /// Sets the `If-Match` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(CLEAR_SITE_DATA, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        CLEAR_SITE_DATA
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
                0 => write!(output, r#""*""#).unwrap(),
                _ => write!(output, r#", "*""#).unwrap(),
            };
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Push a directive into the list of entries.
    pub fn push(&mut self, directive: impl Into<ClearDirective>) {
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

impl IntoIterator for ClearSiteData {
    type Item = ClearDirective;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a ClearSiteData {
    type Item = &'a ClearDirective;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut ClearSiteData {
    type Item = &'a mut ClearDirective;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `ClearSiteData`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<ClearDirective>,
}

impl Iterator for IntoIter {
    type Item = ClearDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `ClearSiteData`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, ClearDirective>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a ClearDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `ClearSiteData`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, ClearDirective>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut ClearDirective;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for ClearSiteData {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for ClearSiteData {
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
    use crate::cache::{ClearDirective, ClearSiteData};
    use crate::Response;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut entries = ClearSiteData::new();
        entries.push(ClearDirective::Cache);
        entries.push(ClearDirective::Cookies);

        let mut res = Response::new(200);
        entries.apply(&mut res);

        let entries = ClearSiteData::from_headers(res)?.unwrap();
        let mut entries = entries.iter();
        assert_eq!(entries.next().unwrap(), &ClearDirective::Cache);
        assert_eq!(entries.next().unwrap(), &ClearDirective::Cookies);
        Ok(())
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut entries = ClearSiteData::new();
        entries.push(ClearDirective::Cache);
        entries.set_wildcard(true);

        let mut res = Response::new(200);
        entries.apply(&mut res);

        let entries = ClearSiteData::from_headers(res)?.unwrap();
        assert!(entries.wildcard());
        let mut entries = entries.iter();
        assert_eq!(entries.next().unwrap(), &ClearDirective::Cache);
        Ok(())
    }

    #[test]
    fn parse_quotes_correctly() -> crate::Result<()> {
        let mut res = Response::new(200);
        res.insert_header("clear-site-data", r#""cookies""#);

        let entries = ClearSiteData::from_headers(res)?.unwrap();
        assert!(!entries.wildcard());
        let mut entries = entries.iter();
        assert_eq!(entries.next().unwrap(), &ClearDirective::Cookies);

        let mut res = Response::new(200);
        res.insert_header("clear-site-data", r#""*""#);

        let entries = ClearSiteData::from_headers(res)?.unwrap();
        assert!(entries.wildcard());
        let mut entries = entries.iter();
        assert_eq!(entries.next(), None);
        Ok(())
    }
}
