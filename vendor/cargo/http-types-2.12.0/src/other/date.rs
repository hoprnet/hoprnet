use crate::headers::{HeaderName, HeaderValue, Headers, DATE};
use crate::utils::HttpDate;

use std::time::SystemTime;

/// The date and time at which the message originated.
///
/// # Specifications
///
/// - [RFC 7231, section 7.1.1.2: Date](https://tools.ietf.org/html/rfc7231#section-7.1.1.2)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::other::Date;
///
/// use std::time::{Duration, SystemTime};
///
/// let now = SystemTime::now();
/// let date = Date::new(now);
///
/// let mut res = Response::new(200);
/// date.apply(&mut res);
///
/// let date = Date::from_headers(res)?.unwrap();
///
/// // Validate we're within 1 second accurate of the system time.
/// assert!(now.duration_since(date.into())? <= Duration::from_secs(1));
/// #
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct Date {
    at: SystemTime,
}

impl Date {
    /// Create a new instance.
    pub fn new(at: SystemTime) -> Self {
        Self { at }
    }

    /// Create a new instance with the date set to now.
    pub fn now() -> Self {
        Self {
            at: SystemTime::now(),
        }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(DATE) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If we successfully parsed the header then there's always at least one
        // entry. We want the last entry.
        let value = headers.iter().last().unwrap();
        let date: HttpDate = value
            .as_str()
            .trim()
            .parse()
            .map_err(|mut e: crate::Error| {
                e.set_status(400);
                e
            })?;
        let at = date.into();
        Ok(Some(Self { at }))
    }

    /// Sets the header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(self.name(), self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        DATE
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let date: HttpDate = self.at.into();
        let output = format!("{}", date);

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }
}

impl From<Date> for SystemTime {
    fn from(date: Date) -> Self {
        date.at
    }
}

impl From<SystemTime> for Date {
    fn from(time: SystemTime) -> Self {
        Self { at: time }
    }
}

impl PartialEq<SystemTime> for Date {
    fn eq(&self, other: &SystemTime) -> bool {
        &self.at == other
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;
    use std::time::Duration;

    #[test]
    fn smoke() -> crate::Result<()> {
        let now = SystemTime::now();
        let date = Date::new(now);

        let mut headers = Headers::new();
        date.apply(&mut headers);

        let date = Date::from_headers(headers)?.unwrap();

        // Validate we're within 1 second accurate of the system time.
        assert!(now.duration_since(date.into())? <= Duration::from_secs(1));
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(DATE, "<nori ate the tag. yum.>");
        let err = Date::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
