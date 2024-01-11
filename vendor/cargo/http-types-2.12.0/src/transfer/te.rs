use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, ACCEPT_ENCODING};
use crate::transfer::{Encoding, EncodingProposal, TransferEncoding};
use crate::utils::sort_by_weight;
use crate::{Error, StatusCode};

use std::fmt::{self, Debug, Write};
use std::option;
use std::slice;

/// Client header advertising the transfer encodings the user agent is willing to
/// accept.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/TE)
///
/// # Specifications
///
/// - [RFC 7230, section 4.3: TE](https://tools.ietf.org/html/rfc7230#section-4.3)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::transfer::{TE, TransferEncoding, Encoding, EncodingProposal};
/// use http_types::Response;
///
/// let mut te = TE::new();
/// te.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);
/// te.push(EncodingProposal::new(Encoding::Gzip, Some(0.4))?);
/// te.push(EncodingProposal::new(Encoding::Identity, None)?);
///
/// let mut res = Response::new(200);
/// let encoding = te.negotiate(&[Encoding::Brotli, Encoding::Gzip])?;
/// encoding.apply(&mut res);
///
/// assert_eq!(res["Content-Encoding"], "br");
/// #
/// # Ok(()) }
/// ```
#[allow(clippy::upper_case_acronyms)]
pub struct TE {
    wildcard: bool,
    entries: Vec<EncodingProposal>,
}

impl TE {
    /// Create a new instance of `TE`.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            wildcard: false,
        }
    }

    /// Create an instance of `TE` from a `Headers` instance.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let headers = match headers.as_ref().get(ACCEPT_ENCODING) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut wildcard = false;

        for value in headers {
            for part in value.as_str().trim().split(',') {
                let part = part.trim();

                // Handle empty strings, and wildcard directives.
                if part.is_empty() {
                    continue;
                } else if part == "*" {
                    wildcard = true;
                    continue;
                }

                // Try and parse a directive from a str. If the directive is
                // unkown we skip it.
                if let Some(entry) = EncodingProposal::from_str(part)? {
                    entries.push(entry);
                }
            }
        }

        Ok(Some(Self { entries, wildcard }))
    }

    /// Push a directive into the list of entries.
    pub fn push(&mut self, prop: impl Into<EncodingProposal>) {
        self.entries.push(prop.into());
    }

    /// Returns `true` if a wildcard directive was passed.
    pub fn wildcard(&self) -> bool {
        self.wildcard
    }

    /// Set the wildcard directive.
    pub fn set_wildcard(&mut self, wildcard: bool) {
        self.wildcard = wildcard
    }

    /// Sort the header directives by weight.
    ///
    /// Headers with a higher `q=` value will be returned first. If two
    /// directives have the same weight, the directive that was declared later
    /// will be returned first.
    pub fn sort(&mut self) {
        sort_by_weight(&mut self.entries);
    }

    /// Determine the most suitable `Transfer-Encoding` encoding.
    ///
    /// # Errors
    ///
    /// If no suitable encoding is found, an error with the status of `406` will be returned.
    pub fn negotiate(&mut self, available: &[Encoding]) -> crate::Result<TransferEncoding> {
        // Start by ordering the encodings.
        self.sort();

        // Try and find the first encoding that matches.
        for encoding in &self.entries {
            if available.contains(encoding) {
                return Ok(encoding.into());
            }
        }

        // If no encoding matches and wildcard is set, send whichever encoding we got.
        if self.wildcard {
            if let Some(encoding) = available.iter().next() {
                return Ok(encoding.into());
            }
        }

        let mut err = Error::new_adhoc("No suitable Transfer-Encoding found");
        err.set_status(StatusCode::NotAcceptable);
        Err(err)
    }

    /// Sets the `Accept-Encoding` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(ACCEPT_ENCODING, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        ACCEPT_ENCODING
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, directive) in self.entries.iter().enumerate() {
            let directive: HeaderValue = (*directive).into();
            match n {
                0 => write!(output, "{}", directive).unwrap(),
                _ => write!(output, ", {}", directive).unwrap(),
            };
        }

        if self.wildcard {
            match output.len() {
                0 => write!(output, "*").unwrap(),
                _ => write!(output, ", *").unwrap(),
            }
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// An iterator visiting all entries.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// An iterator visiting all entries.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.entries.iter_mut(),
        }
    }
}

impl IntoIterator for TE {
    type Item = EncodingProposal;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a TE {
    type Item = &'a EncodingProposal;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut TE {
    type Item = &'a mut EncodingProposal;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `TE`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<EncodingProposal>,
}

impl Iterator for IntoIter {
    type Item = EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `TE`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, EncodingProposal>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `TE`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, EncodingProposal>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for TE {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for TE {
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
    use super::*;
    use crate::transfer::Encoding;
    use crate::Response;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(Encoding::Gzip);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = TE::from_headers(headers)?.unwrap();
        assert_eq!(accept.iter().next().unwrap(), Encoding::Gzip);
        Ok(())
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = TE::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        Ok(())
    }

    #[test]
    fn wildcard_and_header() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(Encoding::Gzip);
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = TE::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        assert_eq!(accept.iter().next().unwrap(), Encoding::Gzip);
        Ok(())
    }

    #[test]
    fn iter() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(Encoding::Gzip);
        accept.push(Encoding::Brotli);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = TE::from_headers(headers)?.unwrap();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), Encoding::Gzip);
        assert_eq!(accept.next().unwrap(), Encoding::Brotli);
        Ok(())
    }

    #[test]
    fn reorder_based_on_weight() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(EncodingProposal::new(Encoding::Gzip, Some(0.4))?);
        accept.push(EncodingProposal::new(Encoding::Identity, None)?);
        accept.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let mut accept = TE::from_headers(headers)?.unwrap();
        accept.sort();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), Encoding::Brotli);
        assert_eq!(accept.next().unwrap(), Encoding::Gzip);
        assert_eq!(accept.next().unwrap(), Encoding::Identity);
        Ok(())
    }

    #[test]
    fn reorder_based_on_weight_and_location() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(EncodingProposal::new(Encoding::Identity, None)?);
        accept.push(EncodingProposal::new(Encoding::Gzip, None)?);
        accept.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);

        let mut res = Response::new(200);
        accept.apply(&mut res);

        let mut accept = TE::from_headers(res)?.unwrap();
        accept.sort();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), Encoding::Brotli);
        assert_eq!(accept.next().unwrap(), Encoding::Gzip);
        assert_eq!(accept.next().unwrap(), Encoding::Identity);
        Ok(())
    }

    #[test]
    fn negotiate() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);
        accept.push(EncodingProposal::new(Encoding::Gzip, Some(0.4))?);
        accept.push(EncodingProposal::new(Encoding::Identity, None)?);

        assert_eq!(
            accept.negotiate(&[Encoding::Brotli, Encoding::Gzip])?,
            Encoding::Brotli,
        );
        Ok(())
    }

    #[test]
    fn negotiate_not_acceptable() -> crate::Result<()> {
        let mut accept = TE::new();
        let err = accept.negotiate(&[Encoding::Gzip]).unwrap_err();
        assert_eq!(err.status(), 406);

        let mut accept = TE::new();
        accept.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);
        let err = accept.negotiate(&[Encoding::Gzip]).unwrap_err();
        assert_eq!(err.status(), 406);
        Ok(())
    }

    #[test]
    fn negotiate_wildcard() -> crate::Result<()> {
        let mut accept = TE::new();
        accept.push(EncodingProposal::new(Encoding::Brotli, Some(0.8))?);
        accept.set_wildcard(true);

        assert_eq!(accept.negotiate(&[Encoding::Gzip])?, Encoding::Gzip);
        Ok(())
    }
}
