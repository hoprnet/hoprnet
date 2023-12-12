use std::fmt;
use uuid::Uuid;

/// Header - `X-Span-ID` - used to track a request through a chain of microservices.
pub const X_SPAN_ID: &str = "X-Span-ID";

/// Wrapper for a string being used as an X-Span-ID.
#[derive(Debug, Clone)]
pub struct XSpanIdString(pub String);

impl XSpanIdString {
    /// Extract an X-Span-ID from a request header if present, and if not
    /// generate a new one.
    pub fn get_or_generate<T>(req: &hyper::Request<T>) -> Self {
        let x_span_id = req.headers().get(X_SPAN_ID);

        x_span_id
            .and_then(|x| x.to_str().ok())
            .map(|x| XSpanIdString(x.to_string()))
            .unwrap_or_default()
    }
}

impl Default for XSpanIdString {
    fn default() -> Self {
        XSpanIdString(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for XSpanIdString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
