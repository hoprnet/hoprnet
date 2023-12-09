use crate::headers::{HeaderName, HeaderValue, Headers, SOURCE_MAP};
use crate::{bail_status as bail, Status, Url};

use std::convert::TryInto;

/// Links to a file that maps transformed source to the original source.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/SourceMap)
///
/// # Specifications
///
/// - [Source Map Revision 3](https://sourcemaps.info/spec.html)
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::{Response, Url};
/// use http_types::other::SourceMap;
///
/// let source_map = SourceMap::new(Url::parse("https://example.net/")?);
///
/// let mut res = Response::new(200);
/// source_map.apply(&mut res);
///
/// let base_url = Url::parse("https://example.net/")?;
/// let source_map = SourceMap::from_headers(base_url, res)?.unwrap();
/// assert_eq!(source_map.location(), &Url::parse("https://example.net/")?);
/// #
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct SourceMap {
    location: Url,
}

impl SourceMap {
    /// Create a new instance of `SourceMap` header.
    pub fn new(location: Url) -> Self {
        Self { location }
    }

    /// Create a new instance from headers.
    pub fn from_headers<U>(base_url: U, headers: impl AsRef<Headers>) -> crate::Result<Option<Self>>
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        let headers = match headers.as_ref().get(SOURCE_MAP) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If we successfully parsed the header then there's always at least one
        // entry. We want the last entry.
        let header_value = headers.iter().last().unwrap();

        let url = match Url::parse(header_value.as_str()) {
            Ok(url) => url,
            Err(_) => match base_url.try_into() {
                Ok(base_url) => base_url.join(header_value.as_str().trim()).status(500)?,
                Err(_) => bail!(500, "Invalid base url provided"),
            },
        };

        Ok(Some(Self { location: url }))
    }

    /// Sets the header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(self.name(), self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        SOURCE_MAP
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let output = self.location.to_string();

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Get the url.
    pub fn location(&self) -> &Url {
        &self.location
    }

    /// Set the url.
    pub fn set_location<U>(&mut self, location: U) -> Result<(), U::Error>
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        self.location = location.try_into()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;

    #[test]
    fn smoke() -> crate::Result<()> {
        let source_map = SourceMap::new(Url::parse("https://example.net/test.json")?);

        let mut headers = Headers::new();
        source_map.apply(&mut headers);

        let base_url = Url::parse("https://example.net/")?;
        let source_map = SourceMap::from_headers(base_url, headers)?.unwrap();
        assert_eq!(
            source_map.location(),
            &Url::parse("https://example.net/test.json")?
        );
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(SOURCE_MAP, "htt://<nori ate the tag. yum.>");
        let err = SourceMap::from_headers(Url::parse("https://example.net").unwrap(), headers)
            .unwrap_err();
        assert_eq!(err.status(), 500);
    }

    #[test]
    fn fallback_works() -> crate::Result<()> {
        let mut headers = Headers::new();
        headers.insert(SOURCE_MAP, "/test.json");

        let base_url = Url::parse("https://fallback.net/")?;
        let source_map = SourceMap::from_headers(base_url, headers)?.unwrap();
        assert_eq!(
            source_map.location(),
            &Url::parse("https://fallback.net/test.json")?
        );

        let mut headers = Headers::new();
        headers.insert(SOURCE_MAP, "https://example.com/test.json");

        let base_url = Url::parse("https://fallback.net/")?;
        let source_map = SourceMap::from_headers(base_url, headers)?.unwrap();
        assert_eq!(
            source_map.location(),
            &Url::parse("https://example.com/test.json")?
        );
        Ok(())
    }
}
