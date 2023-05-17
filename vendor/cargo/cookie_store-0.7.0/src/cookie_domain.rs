use std;

use cookie::Cookie as RawCookie;
use idna;
use publicsuffix;
use serde::{Deserialize, Serialize};
use try_from::TryFrom;
use url::{Host, Url};

use crate::utils::is_host_name;
use crate::CookieError;

pub fn is_match(domain: &str, request_url: &Url) -> bool {
    CookieDomain::try_from(domain)
        .map(|domain| domain.matches(request_url))
        .unwrap_or(false)
}

/// The domain of a `Cookie`
#[derive(PartialEq, Eq, Clone, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CookieDomain {
    /// No Domain attribute in Set-Cookie header
    HostOnly(String),
    /// Domain attribute from Set-Cookie header
    Suffix(String),
    /// Domain attribute was not present in the Set-Cookie header
    NotPresent,
    /// Domain attribute-value was empty; technically undefined behavior, but suggested that this
    /// be treated as invalid
    Empty,
}

// 5.1.3.  Domain Matching
// A string domain-matches a given domain string if at least one of the
// following conditions hold:
//
// o  The domain string and the string are identical.  (Note that both
//    the domain string and the string will have been canonicalized to
//    lower case at this point.)
//
// o  All of the following conditions hold:
//
//    *  The domain string is a suffix of the string.
//
//    *  The last character of the string that is not included in the
//       domain string is a %x2E (".") character.
//
//    *  The string is a host name (i.e., not an IP address).
/// The concept of a domain match per [IETF RFC6265 Section
/// 5.1.3](http://tools.ietf.org/html/rfc6265#section-5.1.3)
impl CookieDomain {
    /// Get the CookieDomain::HostOnly variant based on `request_url`. This is the effective behavior of
    /// setting the domain-attribute to empty
    pub fn host_only(request_url: &Url) -> Result<CookieDomain, CookieError> {
        request_url
            .host()
            .ok_or(CookieError::NonRelativeScheme)
            .map(|h| match h {
                Host::Domain(d) => CookieDomain::HostOnly(d.into()),
                Host::Ipv4(addr) => CookieDomain::HostOnly(format!("{}", addr)),
                Host::Ipv6(addr) => CookieDomain::HostOnly(format!("[{}]", addr)),
            })
    }

    /// Tests if the given `url::Url` meets the domain-match criteria
    pub fn matches(&self, request_url: &Url) -> bool {
        if let Some(url_host) = request_url.host_str() {
            match *self {
                CookieDomain::HostOnly(ref host) => host == url_host,
                CookieDomain::Suffix(ref suffix) => {
                    suffix == url_host
                        || (is_host_name(url_host)
                            && url_host.ends_with(suffix)
                            && url_host[(url_host.len() - suffix.len() - 1)..].starts_with('.'))
                }
                CookieDomain::NotPresent | CookieDomain::Empty => false, // nothing can match the Empty case
            }
        } else {
            false // not a matchable scheme
        }
    }

    /// Tests if the given `url::Url` has a request-host identical to the domain attribute
    pub fn host_is_identical(&self, request_url: &Url) -> bool {
        if let Some(url_host) = request_url.host_str() {
            match *self {
                CookieDomain::HostOnly(ref host) => host == url_host,
                CookieDomain::Suffix(ref suffix) => suffix == url_host,
                CookieDomain::NotPresent | CookieDomain::Empty => false, // nothing can match the Empty case
            }
        } else {
            false // not a matchable scheme
        }
    }

    /// Tests if the domain-attribute is a public suffix as indicated by the provided
    /// `publicsuffix::List`.
    pub fn is_public_suffix(&self, psl: &publicsuffix::List) -> bool {
        if let Some(domain) = self.as_cow() {
            // NB: a failure to parse the domain for publicsuffix usage probably indicates
            // an over-all malformed Domain attribute. However, for the purposes of this test
            // it suffices to indicate that the domain-attribute is not a public suffix, so we
            // discard any such error via `.ok()`
            psl.parse_domain(&domain)
                .ok()
                .and_then(|d| d.suffix().map(|d| d == domain))
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get a borrowed string representation of the domain. For `Empty` and `NotPresent` variants,
    /// `None` shall be returned;
    pub fn as_cow(&self) -> Option<std::borrow::Cow<'_, str>> {
        match *self {
            CookieDomain::HostOnly(ref s) | CookieDomain::Suffix(ref s) => {
                Some(std::borrow::Cow::Borrowed(s))
            }
            CookieDomain::Empty | CookieDomain::NotPresent => None,
        }
    }
}

/// Construct a `CookieDomain::Suffix` from a string, stripping a single leading '.' if present.
/// If the source string is empty, returns the `CookieDomain::Empty` variant.
impl<'a> TryFrom<&'a str> for CookieDomain {
    type Err = failure::Error;
    fn try_from(value: &str) -> Result<CookieDomain, Self::Err> {
        idna::domain_to_ascii(value.trim())
            .map_err(super::IdnaErrors::from)
            .map_err(failure::Error::from)
            .map(|domain| {
                if domain.is_empty() || "." == domain {
                    CookieDomain::Empty
                } else if domain.starts_with('.') {
                    CookieDomain::Suffix(String::from(&domain[1..]))
                } else {
                    CookieDomain::Suffix(domain)
                }
            })
    }
}

/// Construct a `CookieDomain::Suffix` from a `cookie::Cookie`, which handles stripping a leading
/// '.' for us. If the cookie.domain is None or an empty string, the `CookieDomain::Empty` variant
/// is returned.
/// __NOTE__: `cookie::Cookie` domain values already have the leading '.' stripped. To avoid
/// performing this step twice, the `From<&cookie::Cookie>` impl should be used,
/// instead of passing `cookie.domain` to the `From<&str>` impl.
impl<'a, 'c> TryFrom<&'a RawCookie<'c>> for CookieDomain {
    type Err = failure::Error;
    fn try_from(cookie: &'a RawCookie<'c>) -> Result<CookieDomain, Self::Err> {
        if let Some(domain) = cookie.domain() {
            idna::domain_to_ascii(domain.trim())
                .map_err(super::IdnaErrors::from)
                .map_err(failure::Error::from)
                .map(|domain| {
                    if domain.is_empty() {
                        CookieDomain::Empty
                    } else {
                        CookieDomain::Suffix(domain)
                    }
                })
        } else {
            Ok(CookieDomain::NotPresent)
        }
    }
}

impl<'a> From<&'a CookieDomain> for String {
    fn from(c: &'a CookieDomain) -> String {
        match *c {
            CookieDomain::HostOnly(ref h) => h.to_owned(),
            CookieDomain::Suffix(ref s) => s.to_owned(),
            CookieDomain::Empty | CookieDomain::NotPresent => "".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use cookie::Cookie as RawCookie;
    use try_from::TryFrom;
    use url::Url;

    use super::CookieDomain;
    use crate::utils::test::*;

    #[inline]
    fn matches(expected: bool, cookie_domain: &CookieDomain, url: &str) {
        let url = Url::parse(url).unwrap();
        assert!(
            expected == cookie_domain.matches(&url),
            "cookie_domain: {:?} url: {:?}, url.host_str(): {:?}",
            cookie_domain,
            url,
            url.host_str()
        );
    }

    #[inline]
    fn variants(expected: bool, cookie_domain: &CookieDomain, url: &str) {
        matches(expected, cookie_domain, url);
        matches(expected, cookie_domain, &format!("{}/", url));
        matches(expected, cookie_domain, &format!("{}:8080", url));
        matches(expected, cookie_domain, &format!("{}/foo/bar", url));
        matches(expected, cookie_domain, &format!("{}:8080/foo/bar", url));
    }

    #[test]
    fn matches_hostonly() {
        {
            let url = url("http://example.com");
            // HostOnly must be an identical string match, and may be an IP address
            // or a hostname
            let host_name = CookieDomain::host_only(&url).expect("unable to parse domain");
            matches(false, &host_name, "data:nonrelative");
            variants(true, &host_name, "http://example.com");
            variants(false, &host_name, "http://example.org");
            // per RFC6265:
            //    WARNING: Some existing user agents treat an absent Domain
            //      attribute as if the Domain attribute were present and contained
            //      the current host name.  For example, if example.com returns a Set-
            //      Cookie header without a Domain attribute, these user agents will
            //      erroneously send the cookie to www.example.com as well.
            variants(false, &host_name, "http://foo.example.com");
            variants(false, &host_name, "http://127.0.0.1");
            variants(false, &host_name, "http://[::1]");
        }

        {
            let url = url("http://127.0.0.1");
            let ip4 = CookieDomain::host_only(&url).expect("unable to parse Ipv4");
            matches(false, &ip4, "data:nonrelative");
            variants(true, &ip4, "http://127.0.0.1");
            variants(false, &ip4, "http://[::1]");
        }

        {
            let url = url("http://[::1]");
            let ip6 = CookieDomain::host_only(&url).expect("unable to parse Ipv6");
            matches(false, &ip6, "data:nonrelative");
            variants(false, &ip6, "http://127.0.0.1");
            variants(true, &ip6, "http://[::1]");
        }
    }

    #[test]
    fn from_strs() {
        assert_eq!(
            CookieDomain::Empty,
            CookieDomain::try_from("").expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Empty,
            CookieDomain::try_from(".").expect("unable to parse domain")
        );
        // per [IETF RFC6265 Section 5.2.3](https://tools.ietf.org/html/rfc6265#section-5.2.3)
        //If the first character of the attribute-value string is %x2E ("."):
        //
        //Let cookie-domain be the attribute-value without the leading %x2E
        //(".") character.
        assert_eq!(
            CookieDomain::Suffix(String::from(".")),
            CookieDomain::try_from("..").expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Suffix(String::from("example.com")),
            CookieDomain::try_from("example.com").expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Suffix(String::from("example.com")),
            CookieDomain::try_from(".example.com").expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Suffix(String::from(".example.com")),
            CookieDomain::try_from("..example.com").expect("unable to parse domain")
        );
    }

    #[test]
    fn from_raw_cookie() {
        fn raw_cookie(s: &str) -> RawCookie<'_> {
            RawCookie::parse(s).unwrap()
        }
        assert_eq!(
            CookieDomain::NotPresent,
            CookieDomain::try_from(&raw_cookie("cookie=value")).expect("unable to parse domain")
        );
        // cookie::Cookie handles this (cookie.domain == None)
        assert_eq!(
            CookieDomain::NotPresent,
            CookieDomain::try_from(&raw_cookie("cookie=value; Domain="))
                .expect("unable to parse domain")
        );
        // cookie::Cookie does not handle this (empty after stripping leading dot)
        assert_eq!(
            CookieDomain::Empty,
            CookieDomain::try_from(&raw_cookie("cookie=value; Domain=."))
                .expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Suffix(String::from("example.com")),
            CookieDomain::try_from(&raw_cookie("cookie=value; Domain=.example.com"))
                .expect("unable to parse domain")
        );
        assert_eq!(
            CookieDomain::Suffix(String::from("example.com")),
            CookieDomain::try_from(&raw_cookie("cookie=value; Domain=example.com"))
                .expect("unable to parse domain")
        );
    }

    #[test]
    fn matches_suffix() {
        {
            let suffix = CookieDomain::try_from("example.com").expect("unable to parse domain");
            variants(true, &suffix, "http://example.com"); //  exact match
            variants(true, &suffix, "http://foo.example.com"); //  suffix match
            variants(false, &suffix, "http://example.org"); //  no match
            variants(false, &suffix, "http://xample.com"); //  request is the suffix, no match
            variants(false, &suffix, "http://fooexample.com"); //  suffix, but no "." b/w foo and example, no match
        }

        {
            // strip leading dot
            let suffix = CookieDomain::try_from(".example.com").expect("unable to parse domain");
            variants(true, &suffix, "http://example.com");
            variants(true, &suffix, "http://foo.example.com");
            variants(false, &suffix, "http://example.org");
            variants(false, &suffix, "http://xample.com");
            variants(false, &suffix, "http://fooexample.com");
        }

        {
            // only first leading dot is stripped
            let suffix = CookieDomain::try_from("..example.com").expect("unable to parse domain");
            variants(true, &suffix, "http://.example.com");
            variants(true, &suffix, "http://foo..example.com");
            variants(false, &suffix, "http://example.com");
            variants(false, &suffix, "http://foo.example.com");
            variants(false, &suffix, "http://example.org");
            variants(false, &suffix, "http://xample.com");
            variants(false, &suffix, "http://fooexample.com");
        }

        {
            // an exact string match, although an IP is specified
            let suffix = CookieDomain::try_from("127.0.0.1").expect("unable to parse Ipv4");
            variants(true, &suffix, "http://127.0.0.1");
        }

        {
            // an exact string match, although an IP is specified
            let suffix = CookieDomain::try_from("[::1]").expect("unable to parse Ipv6");
            variants(true, &suffix, "http://[::1]");
        }

        {
            // non-identical suffix match only works for host names (i.e. not IPs)
            let suffix = CookieDomain::try_from("0.0.1").expect("unable to parse Ipv4");
            variants(false, &suffix, "http://127.0.0.1");
        }
    }
}

#[cfg(test)]
mod serde_tests {
    use serde_json;
    use try_from::TryFrom;

    use crate::cookie_domain::CookieDomain;
    use crate::utils::test::*;

    fn encode_decode(cd: &CookieDomain, exp_json: &str) {
        let encoded = serde_json::to_string(cd).unwrap();
        assert!(
            exp_json == encoded,
            "expected: '{}'\n encoded: '{}'",
            exp_json,
            encoded
        );
        let decoded: CookieDomain = serde_json::from_str(&encoded).unwrap();
        assert!(
            *cd == decoded,
            "expected: '{:?}'\n decoded: '{:?}'",
            cd,
            decoded
        );
    }

    #[test]
    fn serde() {
        let url = url("http://example.com");
        encode_decode(
            &CookieDomain::host_only(&url).expect("cannot parse domain"),
            "{\"HostOnly\":\"example.com\"}",
        );
        encode_decode(
            &CookieDomain::try_from(".example.com").expect("cannot parse domain"),
            "{\"Suffix\":\"example.com\"}",
        );
        encode_decode(&CookieDomain::NotPresent, "\"NotPresent\"");
        encode_decode(&CookieDomain::Empty, "\"Empty\"");
    }
}
