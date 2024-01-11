//! Client header advertising which media types the client is able to understand.

use crate::content::{ContentType, MediaTypeProposal};
use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, ACCEPT};
use crate::utils::sort_by_weight;
use crate::{Error, Mime, StatusCode};

use std::fmt::{self, Debug, Write};
use std::option;
use std::slice;

/// Client header advertising which media types the client is able to understand.
///
/// Using content negotiation, the server then selects one of the proposals, uses
/// it and informs the client of its choice with the `Content-Type` response
/// header. Browsers set adequate values for this header depending on the context
/// where the request is done: when fetching a CSS stylesheet a different value
/// is set for the request than when fetching an image, video or a script.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept)
///
/// # Specifications
///
/// - [RFC 7231, section 5.3.2: Accept](https://tools.ietf.org/html/rfc7231#section-5.3.2)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::content::{Accept, MediaTypeProposal};
/// use http_types::{mime, Response};
///
/// let mut accept = Accept::new();
/// accept.push(MediaTypeProposal::new(mime::HTML, Some(0.8))?);
/// accept.push(MediaTypeProposal::new(mime::XML, Some(0.4))?);
/// accept.push(mime::PLAIN);
///
/// let mut res = Response::new(200);
/// let content_type = accept.negotiate(&[mime::XML])?;
/// content_type.apply(&mut res);
///
/// assert_eq!(res["Content-Type"], "application/xml;charset=utf-8");
/// #
/// # Ok(()) }
/// ```
pub struct Accept {
    wildcard: bool,
    entries: Vec<MediaTypeProposal>,
}

impl Accept {
    /// Create a new instance of `Accept`.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            wildcard: false,
        }
    }

    /// Create an instance of `Accept` from a `Headers` instance.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let headers = match headers.as_ref().get(ACCEPT) {
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
                let entry = MediaTypeProposal::from_str(part)?;
                entries.push(entry);
            }
        }

        Ok(Some(Self { entries, wildcard }))
    }

    /// Push a directive into the list of entries.
    pub fn push(&mut self, prop: impl Into<MediaTypeProposal>) {
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

    /// Determine the most suitable `Content-Type` encoding.
    ///
    /// # Errors
    ///
    /// If no suitable encoding is found, an error with the status of `406` will be returned.
    pub fn negotiate(&mut self, available: &[Mime]) -> crate::Result<ContentType> {
        // Start by ordering the encodings.
        self.sort();

        // Try and find the first encoding that matches.
        for accept in &self.entries {
            if available.contains(accept) {
                return Ok(accept.media_type.clone().into());
            }
        }

        // If no encoding matches and wildcard is set, send whichever encoding we got.
        if self.wildcard {
            if let Some(accept) = available.iter().next() {
                return Ok(accept.clone().into());
            }
        }

        let mut err = Error::new_adhoc("No suitable Content-Type found");
        err.set_status(StatusCode::NotAcceptable);
        Err(err)
    }

    /// Sets the `Accept-Encoding` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(ACCEPT, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        ACCEPT
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

impl IntoIterator for Accept {
    type Item = MediaTypeProposal;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Accept {
    type Item = &'a MediaTypeProposal;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Accept {
    type Item = &'a mut MediaTypeProposal;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `Accept`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<MediaTypeProposal>,
}

impl Iterator for IntoIter {
    type Item = MediaTypeProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `Accept`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, MediaTypeProposal>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a MediaTypeProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `Accept`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, MediaTypeProposal>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut MediaTypeProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for Accept {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for Accept {
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
    use crate::mime;
    use crate::Response;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(mime::HTML);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = Accept::from_headers(headers)?.unwrap();
        assert_eq!(accept.iter().next().unwrap(), mime::HTML);
        Ok(())
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = Accept::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        Ok(())
    }

    #[test]
    fn wildcard_and_header() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(mime::HTML);
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = Accept::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        assert_eq!(accept.iter().next().unwrap(), mime::HTML);
        Ok(())
    }

    #[test]
    fn iter() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(mime::HTML);
        accept.push(mime::XML);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = Accept::from_headers(headers)?.unwrap();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), mime::HTML);
        assert_eq!(accept.next().unwrap(), mime::XML);
        Ok(())
    }

    #[test]
    fn reorder_based_on_weight() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(MediaTypeProposal::new(mime::HTML, Some(0.4))?);
        accept.push(MediaTypeProposal::new(mime::XML, None)?);
        accept.push(MediaTypeProposal::new(mime::PLAIN, Some(0.8))?);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let mut accept = Accept::from_headers(headers)?.unwrap();
        accept.sort();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), mime::PLAIN);
        assert_eq!(accept.next().unwrap(), mime::HTML);
        assert_eq!(accept.next().unwrap(), mime::XML);
        Ok(())
    }

    #[test]
    fn reorder_based_on_weight_and_location() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(MediaTypeProposal::new(mime::HTML, None)?);
        accept.push(MediaTypeProposal::new(mime::XML, None)?);
        accept.push(MediaTypeProposal::new(mime::PLAIN, Some(0.8))?);

        let mut res = Response::new(200);
        accept.apply(&mut res);

        let mut accept = Accept::from_headers(res)?.unwrap();
        accept.sort();
        let mut accept = accept.iter();
        assert_eq!(accept.next().unwrap(), mime::PLAIN);
        assert_eq!(accept.next().unwrap(), mime::XML);
        assert_eq!(accept.next().unwrap(), mime::HTML);
        Ok(())
    }

    #[test]
    fn negotiate() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(MediaTypeProposal::new(mime::HTML, Some(0.4))?);
        accept.push(MediaTypeProposal::new(mime::PLAIN, Some(0.8))?);
        accept.push(MediaTypeProposal::new(mime::XML, None)?);

        assert_eq!(accept.negotiate(&[mime::HTML, mime::XML])?, mime::HTML);
        Ok(())
    }

    #[test]
    fn negotiate_not_acceptable() -> crate::Result<()> {
        let mut accept = Accept::new();
        let err = accept.negotiate(&[mime::JSON]).unwrap_err();
        assert_eq!(err.status(), 406);

        let mut accept = Accept::new();
        accept.push(MediaTypeProposal::new(mime::JSON, Some(0.8))?);
        let err = accept.negotiate(&[mime::XML]).unwrap_err();
        assert_eq!(err.status(), 406);
        Ok(())
    }

    #[test]
    fn negotiate_wildcard() -> crate::Result<()> {
        let mut accept = Accept::new();
        accept.push(MediaTypeProposal::new(mime::JSON, Some(0.8))?);
        accept.set_wildcard(true);

        assert_eq!(accept.negotiate(&[mime::XML])?, mime::XML);
        Ok(())
    }
}
