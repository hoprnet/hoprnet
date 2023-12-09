use std::time::{Duration, SystemTime, SystemTimeError};

use crate::headers::{HeaderName, HeaderValue, Headers, RETRY_AFTER};
use crate::utils::{fmt_http_date, parse_http_date};

/// Indicate how long the user agent should wait before making a follow-up request.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After)
///
/// # Specifications
///
/// - [RFC 7231, section 3.1.4.2: Retry-After](https://tools.ietf.org/html/rfc7231#section-3.1.4.2)
///
/// # Examples
///
/// ```no_run
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::other::RetryAfter;
/// use http_types::Response;
/// use std::time::{SystemTime, Duration};
/// use async_std::task;
///
/// let retry = RetryAfter::new(Duration::from_secs(10));
///
/// let mut headers = Response::new(429);
/// retry.apply(&mut headers);
///
/// // Sleep for the duration, then try the task again.
/// let retry = RetryAfter::from_headers(headers)?.unwrap();
/// task::sleep(retry.duration_since(SystemTime::now())?);
/// #
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetryAfter {
    inner: RetryDirective,
}

#[allow(clippy::len_without_is_empty)]
impl RetryAfter {
    /// Create a new instance from a `Duration`.
    ///
    /// This value will be encoded over the wire as a relative offset in seconds.
    pub fn new(dur: Duration) -> Self {
        Self {
            inner: RetryDirective::Duration(dur),
        }
    }

    /// Create a new instance from a `SystemTime` instant.
    ///
    /// This value will be encoded a specific `Date` over the wire.
    pub fn new_at(at: SystemTime) -> Self {
        Self {
            inner: RetryDirective::SystemTime(at),
        }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let header = match headers.as_ref().get(RETRY_AFTER) {
            Some(headers) => headers.last(),
            None => return Ok(None),
        };

        let inner = match header.as_str().parse::<u64>() {
            Ok(dur) => RetryDirective::Duration(Duration::from_secs(dur)),
            Err(_) => {
                let at = parse_http_date(header.as_str())?;
                RetryDirective::SystemTime(at)
            }
        };
        Ok(Some(Self { inner }))
    }

    /// Returns the amount of time elapsed from an earlier point in time.
    ///
    /// # Errors
    ///
    /// This may return an error if the `earlier` time was after the current time.
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        let at = match self.inner {
            RetryDirective::Duration(dur) => SystemTime::now() + dur,
            RetryDirective::SystemTime(at) => at,
        };

        at.duration_since(earlier)
    }

    /// Sets the header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(self.name(), self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        RETRY_AFTER
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let output = match self.inner {
            RetryDirective::Duration(dur) => format!("{}", dur.as_secs()),
            RetryDirective::SystemTime(at) => fmt_http_date(at),
        };
        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }
}

impl From<RetryAfter> for SystemTime {
    fn from(retry_after: RetryAfter) -> Self {
        match retry_after.inner {
            RetryDirective::Duration(dur) => SystemTime::now() + dur,
            RetryDirective::SystemTime(at) => at,
        }
    }
}

/// What value are we decoding into?
///
/// This value is intionally never exposes; all end-users want is a `Duration`
/// value that tells them how long to wait for before trying again.
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum RetryDirective {
    Duration(Duration),
    SystemTime(SystemTime),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;

    #[test]
    fn smoke() -> crate::Result<()> {
        let retry = RetryAfter::new(Duration::from_secs(10));

        let mut headers = Headers::new();
        retry.apply(&mut headers);

        // `SystemTime::now` uses sub-second precision which means there's some
        // offset that's not encoded.
        let now = SystemTime::now();
        let retry = RetryAfter::from_headers(headers)?.unwrap();
        assert_eq!(
            retry.duration_since(now)?.as_secs(),
            Duration::from_secs(10).as_secs()
        );
        Ok(())
    }

    #[test]
    fn new_at() -> crate::Result<()> {
        let now = SystemTime::now();
        let retry = RetryAfter::new_at(now + Duration::from_secs(10));

        let mut headers = Headers::new();
        retry.apply(&mut headers);

        // `SystemTime::now` uses sub-second precision which means there's some
        // offset that's not encoded.
        let retry = RetryAfter::from_headers(headers)?.unwrap();
        let delta = retry.duration_since(now)?;
        assert!(delta >= Duration::from_secs(9));
        assert!(delta <= Duration::from_secs(10));
        Ok(())
    }
}
