use crate::auth::AuthenticationScheme;
use crate::bail_status as bail;
use crate::headers::{HeaderName, HeaderValue, Headers, WWW_AUTHENTICATE};

/// Define the authentication method that should be used to gain access to a
/// resource.
///
/// # Specifications
///
/// - [RFC 7235, section 4.1: WWW-Authenticate](https://tools.ietf.org/html/rfc7235#section-4.1)
///
/// # Implementation Notes
///
/// This implementation only encodes and parses a single authentication method,
/// further authorization methods are ignored. It also always passes the utf-8 encoding flag.
///
/// # Examples
///
/// ```
/// # fn main() -> http_types::Result<()> {
/// #
/// use http_types::Response;
/// use http_types::auth::{AuthenticationScheme, WwwAuthenticate};
///
/// let scheme = AuthenticationScheme::Basic;
/// let realm = "Access to the staging site";
/// let authz = WwwAuthenticate::new(scheme, realm.into());
///
/// let mut res = Response::new(200);
/// authz.apply(&mut res);
///
/// let authz = WwwAuthenticate::from_headers(res)?.unwrap();
///
/// assert_eq!(authz.scheme(), AuthenticationScheme::Basic);
/// assert_eq!(authz.realm(), realm);
/// #
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct WwwAuthenticate {
    scheme: AuthenticationScheme,
    realm: String,
}

impl WwwAuthenticate {
    /// Create a new instance of `WwwAuthenticate`.
    pub fn new(scheme: AuthenticationScheme, realm: String) -> Self {
        Self { scheme, realm }
    }

    /// Create a new instance from headers.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let headers = match headers.as_ref().get(WWW_AUTHENTICATE) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        // If we successfully parsed the header then there's always at least one
        // entry. We want the last entry.
        let value = headers.iter().last().unwrap();

        let mut iter = value.as_str().splitn(2, ' ');
        let scheme = iter.next();
        let credential = iter.next();
        let (scheme, realm) = match (scheme, credential) {
            (None, _) => bail!(400, "Could not find scheme"),
            (Some(_), None) => bail!(400, "Could not find realm"),
            (Some(scheme), Some(realm)) => (scheme.parse()?, realm.to_owned()),
        };

        let realm = realm.trim_start();
        let realm = match realm.strip_prefix(r#"realm=""#) {
            Some(realm) => realm,
            None => bail!(400, "realm not found"),
        };

        let mut chars = realm.chars();
        let mut closing_quote = false;
        let realm = (&mut chars)
            .take_while(|c| {
                if c == &'"' {
                    closing_quote = true;
                    false
                } else {
                    true
                }
            })
            .collect();
        if !closing_quote {
            bail!(400, r"Expected a closing quote");
        }

        Ok(Some(Self { scheme, realm }))
    }

    /// Sets the header.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(self.name(), self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        WWW_AUTHENTICATE
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let output = format!(r#"{} realm="{}", charset="UTF-8""#, self.scheme, self.realm);

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// Get the authorization scheme.
    pub fn scheme(&self) -> AuthenticationScheme {
        self.scheme
    }

    /// Set the authorization scheme.
    pub fn set_scheme(&mut self, scheme: AuthenticationScheme) {
        self.scheme = scheme;
    }

    /// Get the authorization realm.
    pub fn realm(&self) -> &str {
        self.realm.as_str()
    }

    /// Set the authorization realm.
    pub fn set_realm(&mut self, realm: String) {
        self.realm = realm;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::Headers;

    #[test]
    fn smoke() -> crate::Result<()> {
        let scheme = AuthenticationScheme::Basic;
        let realm = "Access to the staging site";
        let authz = WwwAuthenticate::new(scheme, realm.into());

        let mut headers = Headers::new();
        authz.apply(&mut headers);

        assert_eq!(
            headers["WWW-Authenticate"],
            r#"Basic realm="Access to the staging site", charset="UTF-8""#
        );

        let authz = WwwAuthenticate::from_headers(headers)?.unwrap();

        assert_eq!(authz.scheme(), AuthenticationScheme::Basic);
        assert_eq!(authz.realm(), realm);
        Ok(())
    }

    #[test]
    fn bad_request_on_parse_error() {
        let mut headers = Headers::new();
        headers.insert(WWW_AUTHENTICATE, "<nori ate the tag. yum.>");
        let err = WwwAuthenticate::from_headers(headers).unwrap_err();
        assert_eq!(err.status(), 400);
    }
}
