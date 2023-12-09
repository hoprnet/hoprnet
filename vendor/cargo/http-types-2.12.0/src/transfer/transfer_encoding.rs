use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, CONTENT_ENCODING};
use crate::transfer::{Encoding, EncodingProposal};

use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::option;

/// The form of encoding used to safely transfer the payload body to the user.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding)
///
/// # Specifications
///
/// - [RFC 7230, section 3.3.1: Transfer-Encoding](https://tools.ietf.org/html/rfc7230#section-3.3.1)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::transfer::{TransferEncoding, Encoding};
/// let mut encoding = TransferEncoding::new(Encoding::Chunked);
///
/// let mut res = Response::new(200);
/// encoding.apply(&mut res);
///
/// let encoding = TransferEncoding::from_headers(res)?.unwrap();
/// assert_eq!(encoding, &Encoding::Chunked);
/// #
/// # Ok(()) }
/// ```
pub struct TransferEncoding {
    inner: Encoding,
}

impl TransferEncoding {
    /// Create a new instance of `CacheControl`.
    pub fn new(encoding: Encoding) -> Self {
        Self { inner: encoding }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(CONTENT_ENCODING) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut inner = None;

        for value in headers {
            if let Some(entry) = Encoding::from_str(value.as_str()) {
                inner = Some(entry);
            }
        }

        let inner = inner.expect("Headers instance with no entries found");
        Ok(Some(Self { inner }))
    }

    /// Sets the `Content-Encoding` header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(CONTENT_ENCODING, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        CONTENT_ENCODING
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        self.inner.into()
    }

    /// Access the encoding kind.
    pub fn encoding(&self) -> Encoding {
        self.inner
    }
}

impl ToHeaderValues for TransferEncoding {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Deref for TransferEncoding {
    type Target = Encoding;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TransferEncoding {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl PartialEq<Encoding> for TransferEncoding {
    fn eq(&self, other: &Encoding) -> bool {
        &self.inner == other
    }
}

impl PartialEq<&Encoding> for TransferEncoding {
    fn eq(&self, other: &&Encoding) -> bool {
        &&self.inner == other
    }
}

impl From<Encoding> for TransferEncoding {
    fn from(encoding: Encoding) -> Self {
        Self { inner: encoding }
    }
}

impl From<&Encoding> for TransferEncoding {
    fn from(encoding: &Encoding) -> Self {
        Self { inner: *encoding }
    }
}

impl From<EncodingProposal> for TransferEncoding {
    fn from(encoding: EncodingProposal) -> Self {
        Self {
            inner: encoding.encoding,
        }
    }
}

impl From<&EncodingProposal> for TransferEncoding {
    fn from(encoding: &EncodingProposal) -> Self {
        Self {
            inner: encoding.encoding,
        }
    }
}

impl Debug for TransferEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
