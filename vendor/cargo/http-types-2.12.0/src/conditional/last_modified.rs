use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, LAST_MODIFIED};
use crate::utils::{fmt_http_date, parse_http_date};

use std::fmt::Debug;
use std::option;
use std::time::SystemTime;

/// The last modification date of a resource.
///
/// # Specifications
///
/// - [RFC 7232, section 2.2: Last-Modified](https://tools.ietf.org/html/rfc7232#section-2.2)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::conditional::LastModified;
/// use std::time::{SystemTime, Duration};
///
/// let time = SystemTime::now() + Duration::from_secs(5 * 60);
/// let last_modified = LastModified::new(time);
///
/// let mut res = Response::new(200);
/// last_modified.apply(&mut res);
///
/// let last_modified = LastModified::from_headers(res)?.unwrap();
///
/// // HTTP dates only have second-precision.
/// let elapsed = time.duration_since(last_modified.modified())?;
/// assert_eq!(elapsed.as_secs(), 0);
/// #
/// # Ok(()) }
/// ```
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LastModified {
    instant: SystemTime,
}

impl LastModified {
    /// Create a new instance of `LastModified`.
    pub fn new(instant: SystemTime) -> Self {
        Self { instant }
    }

    /// Returns the last modification time listed.
    pub fn modified(&self) -> SystemTime {
        self.instant
    }

    /// Create an instance of `LastModified` from a `Headers` instance.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(LAST_MODIFIED) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If we successfully parsed the header then there's always at least one
        // entry. We want the last entry.
        let header = headers.iter().last().unwrap();

        let instant = parse_http_date(header.as_str())?;
        Ok(Some(Self { instant }))
    }

    /// Insert a `HeaderName` + `HeaderValue` pair into a `Headers` instance.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(LAST_MODIFIED, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        LAST_MODIFIED
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let output = fmt_http_date(self.instant);

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }
}

impl ToHeaderValues for LastModified {
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
    use std::time::Duration;

    #[test]
    fn smoke() -> crate::Result<()> {
        let time = SystemTime::now() + Duration::from_secs(5 * 60);
        let last_modified = LastModified::new(time);

        let mut headers = Headers::new();
        last_modified.apply(&mut headers);

        let last_modified = LastModified::from_headers(headers)?.unwrap();

        // HTTP dates only have second-precision
        let elapsed = time.duration_since(last_modified.modified())?;
        assert_eq!(elapsed.as_secs(), 0);
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(LAST_MODIFIED, "<nori ate the tag. yum.>");
        let err = LastModified::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
