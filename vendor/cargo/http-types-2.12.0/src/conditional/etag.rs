use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, ETAG};
use crate::{Error, StatusCode};

use std::fmt::{self, Debug, Display};
use std::option;

/// HTTP Entity Tags.
///
/// ETags provide an ID for a particular resource, enabling clients and servers
/// to reason about caches and make conditional requests.
///
/// # Specifications
///
/// - [RFC 7232, section 2.3: ETag](https://tools.ietf.org/html/rfc7232#section-2.3)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::conditional::ETag;
///
/// let etag = ETag::new("0xcafebeef".to_string());
///
/// let mut res = Response::new(200);
/// etag.apply(&mut res);
///
/// let etag = ETag::from_headers(res)?.unwrap();
/// assert_eq!(etag, ETag::Strong(String::from("0xcafebeef")));
/// #
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ETag {
    /// An ETag using strong validation.
    Strong(String),
    /// An ETag using weak validation.
    Weak(String),
}

impl ETag {
    /// Create a new ETag that uses strong validation.
    pub fn new(s: String) -> Self {
        debug_assert!(!s.contains('\\'), "ETags ought to avoid backslash chars");
        Self::Strong(s)
    }

    /// Create a new ETag that uses weak validation.
    pub fn new_weak(s: String) -> Self {
        debug_assert!(!s.contains('\\'), "ETags ought to avoid backslash chars");
        Self::Weak(s)
    }

    /// Create a new instance from headers.
    ///
    /// Only a single ETag per resource is assumed to exist. If multiple ETag
    /// headers are found the last one is used.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(ETAG) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If a header is returned we can assume at least one exists.
        let s = headers.iter().last().unwrap().as_str();
        Self::from_str(s).map(Some)
    }

    /// Sets the `ETag` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(ETAG, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        ETAG
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let s = self.to_string();
        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(s.into()) }
    }

    /// Returns `true` if the ETag is a `Strong` value.
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Strong(_))
    }

    /// Returns `true` if the ETag is a `Weak` value.
    pub fn is_weak(&self) -> bool {
        matches!(self, Self::Weak(_))
    }

    /// Create an Etag from a string.
    pub(crate) fn from_str(s: &str) -> crate::Result<Self> {
        let mut weak = false;
        let s = match s.strip_prefix("W/") {
            Some(s) => {
                weak = true;
                s
            }
            None => s,
        };

        let s = match s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            Some(s) => s.to_owned(),
            None => {
                return Err(Error::from_str(
                    StatusCode::BadRequest,
                    "Invalid ETag header",
                ))
            }
        };

        if !s
            .bytes()
            .all(|c| c == 0x21 || (0x23..=0x7E).contains(&c) || c >= 0x80)
        {
            return Err(Error::from_str(
                StatusCode::BadRequest,
                "Invalid ETag header",
            ));
        }

        let etag = if weak { Self::Weak(s) } else { Self::Strong(s) };
        Ok(etag)
    }
}

impl Display for ETag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strong(s) => write!(f, r#""{}""#, s),
            Self::Weak(s) => write!(f, r#"W/"{}""#, s),
        }
    }
}

impl ToHeaderValues for ETag {
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
        let etag = ETag::new("0xcafebeef".to_string());

        let mut headers = Headers::new();
        etag.apply(&mut headers);

        let etag = ETag::from_headers(headers)?.unwrap();
        assert_eq!(etag, ETag::Strong(String::from("0xcafebeef")));
        Ok(())
    }

    #[test]
    fn smoke_weak() -> crate::Result<()> {
        let etag = ETag::new_weak("0xcafebeef".to_string());

        let mut headers = Headers::new();
        etag.apply(&mut headers);

        let etag = ETag::from_headers(headers)?.unwrap();
        assert_eq!(etag, ETag::Weak(String::from("0xcafebeef")));
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(ETAG, "<nori ate the tag. yum.>");
        let err = ETag::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }

    #[test]
    fn validate_quotes() {
        assert_entry_err(r#""hello"#, "Invalid ETag header");
        assert_entry_err(r#"hello""#, "Invalid ETag header");
        assert_entry_err(r#"/O"valid content""#, "Invalid ETag header");
        assert_entry_err(r#"/Wvalid content""#, "Invalid ETag header");
    }

    fn assert_entry_err(s: &str, msg: &str) {
        let mut headers = Headers::new();
        headers.insert(ETAG, s);
        let err = ETag::from_headers(headers).unwrap_err();
        assert_eq!(format!("{}", err), msg);
    }

    #[test]
    fn validate_characters() {
        assert_entry_err(r#"""hello""#, "Invalid ETag header");
        assert_entry_err("\"hello\x7F\"", "Invalid ETag header");
    }
}
