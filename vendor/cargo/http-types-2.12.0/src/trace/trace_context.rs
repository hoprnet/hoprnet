use rand::Rng;
use std::fmt;

use crate::headers::{HeaderName, HeaderValue, Headers, TRACEPARENT};
use crate::Status;

/// Extract and apply [Trace-Context](https://w3c.github.io/trace-context/) headers.
///
/// # Specifications
///
/// - [Trace-Context (Working Draft)](https://w3c.github.io/trace-context/)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::trace::TraceContext;
///
/// let mut res = http_types::Response::new(200);
///
/// res.insert_header(
///     "traceparent",
///     "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01"
/// );
///
/// let context = TraceContext::from_headers(&res)?.unwrap();
///
/// let trace_id = u128::from_str_radix("0af7651916cd43dd8448eb211c80319c", 16);
/// let parent_id = u64::from_str_radix("00f067aa0ba902b7", 16);
///
/// assert_eq!(context.trace_id(), trace_id.unwrap());
/// assert_eq!(context.parent_id(), parent_id.ok());
/// assert_eq!(context.sampled(), true);
/// #
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct TraceContext {
    id: u64,
    version: u8,
    trace_id: u128,
    parent_id: Option<u64>,
    flags: u8,
}

impl TraceContext {
    /// Generate a new TraceContext object without a parent.
    ///
    /// By default root TraceContext objects are sampled.
    /// To mark it unsampled, call `context.set_sampled(false)`.
    ///
    /// # Examples
    /// ```
    /// use http_types::trace::TraceContext;
    ///
    /// let context = TraceContext::new();
    ///
    /// assert_eq!(context.parent_id(), None);
    /// assert_eq!(context.sampled(), true);
    /// ```
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            id: rng.gen(),
            version: 0,
            trace_id: rng.gen(),
            parent_id: None,
            flags: 1,
        }
    }

    /// Create and return TraceContext object based on `traceparent` HTTP header.
    ///
    /// # Errors
    ///
    /// This function may error if the header is malformed. An error with a
    /// status code of `400: Bad Request` will be generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> http_types::Result<()> {
    /// #
    /// use http_types::trace::TraceContext;
    ///
    /// let mut res = http_types::Response::new(200);
    /// res.insert_header(
    ///   "traceparent",
    ///   "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01"
    /// );
    ///
    /// let context = TraceContext::from_headers(&res)?.unwrap();
    ///
    /// let trace_id = u128::from_str_radix("0af7651916cd43dd8448eb211c80319c", 16);
    /// let parent_id = u64::from_str_radix("00f067aa0ba902b7", 16);
    ///
    /// assert_eq!(context.trace_id(), trace_id.unwrap());
    /// assert_eq!(context.parent_id(), parent_id.ok());
    /// assert_eq!(context.sampled(), true);
    /// #
    /// # Ok(()) }
    /// ```
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = headers.as_ref();
        let mut rng = rand::thread_rng();

        let traceparent = match headers.get(TRACEPARENT) {
            Some(header) => header,
            None => return Ok(None),
        };
        let parts: Vec<&str> = traceparent.as_str().split('-').collect();

        Ok(Some(Self {
            id: rng.gen(),
            version: u8::from_str_radix(parts[0], 16)?,
            trace_id: u128::from_str_radix(parts[1], 16).status(400)?,
            parent_id: Some(u64::from_str_radix(parts[2], 16).status(400)?),
            flags: u8::from_str_radix(parts[3], 16).status(400)?,
        }))
    }

    /// Add the traceparent header to the http headers
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> http_types::Result<()> {
    /// #
    /// use http_types::trace::TraceContext;
    /// use http_types::{Request, Response, Url, Method};
    ///
    /// let mut req = Request::new(Method::Get, Url::parse("https://example.com").unwrap());
    /// req.insert_header(
    ///   "traceparent",
    ///   "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01"
    /// );
    ///
    /// let parent = TraceContext::from_headers(&req)?.unwrap();
    ///
    /// let mut res = Response::new(200);
    /// parent.apply(&mut res);
    ///
    /// let child = TraceContext::from_headers(&res)?.unwrap();
    ///
    /// assert_eq!(child.version(), parent.version());
    /// assert_eq!(child.trace_id(), parent.trace_id());
    /// assert_eq!(child.parent_id(), Some(parent.id()));
    /// #
    /// # Ok(()) }
    /// ```
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        let headers = headers.as_mut();
        headers.insert(TRACEPARENT, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        TRACEPARENT
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let output = format!("{}", self);
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Generate a child of the current TraceContext and return it.
    ///
    /// The child will have a new randomly genrated `id` and its `parent_id` will be set to the
    /// `id` of this TraceContext.
    pub fn child(&self) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            id: rng.gen(),
            version: self.version,
            trace_id: self.trace_id,
            parent_id: Some(self.id),
            flags: self.flags,
        }
    }

    /// Return the id of the TraceContext.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Return the version of the TraceContext spec used.
    ///
    /// You probably don't need this.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Return the trace id of the TraceContext.
    ///
    /// All children will have the same `trace_id`.
    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    /// Return the id of the parent TraceContext.
    #[inline]
    pub fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }

    /// Returns true if the trace is sampled
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> http_types::Result<()> {
    /// #
    /// use http_types::trace::TraceContext;
    /// use http_types::Response;
    ///
    /// let mut res = Response::new(200);
    /// res.insert_header("traceparent", "00-00000000000000000000000000000001-0000000000000002-01");
    /// let context = TraceContext::from_headers(&res)?.unwrap();
    /// assert_eq!(context.sampled(), true);
    /// #
    /// # Ok(()) }
    /// ```
    pub fn sampled(&self) -> bool {
        (self.flags & 0b00000001) == 1
    }

    /// Change sampled flag
    ///
    /// # Examples
    ///
    /// ```
    /// use http_types::trace::TraceContext;
    ///
    /// let mut context = TraceContext::new();
    /// assert_eq!(context.sampled(), true);
    /// context.set_sampled(false);
    /// assert_eq!(context.sampled(), false);
    /// ```
    pub fn set_sampled(&mut self, sampled: bool) {
        let x = sampled as u8;
        self.flags ^= (x ^ self.flags) & (1 << 0);
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}-{:032x}-{:016x}-{:02x}",
            self.version, self.trace_id, self.id, self.flags
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default() -> crate::Result<()> {
        let mut headers = crate::Headers::new();
        headers.insert(TRACEPARENT, "00-01-deadbeef-00");
        let context = TraceContext::from_headers(&mut headers)?.unwrap();
        assert_eq!(context.version(), 0);
        assert_eq!(context.trace_id(), 1);
        assert_eq!(context.parent_id().unwrap(), 3735928559);
        assert_eq!(context.flags, 0);
        assert!(!context.sampled());
        Ok(())
    }

    #[test]
    fn no_header() {
        let context = TraceContext::new();
        assert_eq!(context.version(), 0);
        assert_eq!(context.parent_id(), None);
        assert_eq!(context.flags, 1);
        assert!(context.sampled());
    }

    #[test]
    fn not_sampled() -> crate::Result<()> {
        let mut headers = crate::Headers::new();
        headers.insert(TRACEPARENT, "00-01-02-00");
        let context = TraceContext::from_headers(&mut headers)?.unwrap();
        assert!(!context.sampled());
        Ok(())
    }

    #[test]
    fn sampled() -> crate::Result<()> {
        let mut headers = crate::Headers::new();
        headers.insert(TRACEPARENT, "00-01-02-01");
        let context = TraceContext::from_headers(&mut headers)?.unwrap();
        assert!(context.sampled());
        Ok(())
    }
}
