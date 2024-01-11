use crate::ensure_eq_status;
use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, EXPECT};

use std::fmt::Debug;
use std::option;

/// HTTP `Expect` header
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect)
///
/// # Specifications
///
/// - [RFC 7231, section 5.1.1: Expect](https://tools.ietf.org/html/rfc7231#section-5.1.1)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::other::Expect;
///
/// let expect = Expect::new();
///
/// let mut res = Response::new(200);
/// expect.apply(&mut res);
///
/// let expect = Expect::from_headers(res)?.unwrap();
/// assert_eq!(expect, Expect::new());
/// #
/// # Ok(()) }
/// ```
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Expect {
    _priv: (),
}

impl Expect {
    /// Create a new instance of `Expect`.
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Create an instance of `Expect` from a `Headers` instance.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(EXPECT) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If we successfully parsed the header then there's always at least one
        // entry. We want the last entry.
        let header = headers.iter().last().unwrap();
        ensure_eq_status!(header, "100-continue", 400, "malformed `Expect` header");

        Ok(Some(Self { _priv: () }))
    }

    /// Insert a `HeaderName` + `HeaderValue` pair into a `Headers` instance.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(EXPECT, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        EXPECT
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let value = "100-continue";
        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(value.into()) }
    }
}

impl ToHeaderValues for Expect {
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
        let expect = Expect::new();

        let mut headers = Headers::new();
        expect.apply(&mut headers);

        let expect = Expect::from_headers(headers)?.unwrap();
        assert_eq!(expect, Expect::new());
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(EXPECT, "<nori ate the tag. yum.>");
        let err = Expect::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
