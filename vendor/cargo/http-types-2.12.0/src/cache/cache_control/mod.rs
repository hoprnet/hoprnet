//! HTTP `Cache-Control` headers.
//!
//! # Specifications
//!
//! - [RFC 8246: HTTP Immutable Responses](https://tools.ietf.org/html/rfc8246)
//! - [RFC 7234: Hypertext Transfer Protocol (HTTP/1.1): Caching](https://tools.ietf.org/html/rfc7234)
//! - [RFC 5861: HTTP Cache-Control Extensions for Stale Content](https://tools.ietf.org/html/rfc5861)

#[allow(clippy::module_inception)]
mod cache_control;
mod cache_directive;

pub use cache_control::CacheControl;
pub use cache_directive::CacheDirective;

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::{Headers, CACHE_CONTROL};

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut entries = CacheControl::new();
        entries.push(CacheDirective::Immutable);
        entries.push(CacheDirective::NoStore);

        let mut headers = Headers::new();
        entries.apply(&mut headers);

        let entries = CacheControl::from_headers(headers)?.unwrap();
        let mut entries = entries.iter();
        assert_eq!(entries.next().unwrap(), &CacheDirective::Immutable);
        assert_eq!(entries.next().unwrap(), &CacheDirective::NoStore);
        Ok(())
    }

    #[test]
    fn ignore_unkonwn_directives() -> crate::Result<()> {
        let mut headers = Headers::new();
        headers.insert(CACHE_CONTROL, "barrel_roll");
        let entries = CacheControl::from_headers(headers)?.unwrap();
        let mut entries = entries.iter();
        assert!(entries.next().is_none());
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(CACHE_CONTROL, "min-fresh=0.9"); // floats are not supported
        let err = CacheControl::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
