use crate::{
    headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, FORWARDED},
    parse_utils::{parse_quoted_string, parse_token},
};
use std::{borrow::Cow, convert::TryFrom, fmt::Write, net::IpAddr};

// these constants are private because they are nonstandard
const X_FORWARDED_FOR: HeaderName = HeaderName::from_lowercase_str("x-forwarded-for");
const X_FORWARDED_PROTO: HeaderName = HeaderName::from_lowercase_str("x-forwarded-proto");
const X_FORWARDED_BY: HeaderName = HeaderName::from_lowercase_str("x-forwarded-by");

/// A rust representation of the [forwarded
/// header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Forwarded<'a> {
    by: Option<Cow<'a, str>>,
    forwarded_for: Vec<Cow<'a, str>>,
    host: Option<Cow<'a, str>>,
    proto: Option<Cow<'a, str>>,
}

impl<'a> Forwarded<'a> {
    /// Attempts to parse a Forwarded from headers (or a request or
    /// response). Builds a borrowed Forwarded by default. To build an
    /// owned Forwarded, use
    /// `Forwarded::from_headers(...).into_owned()`
    ///
    /// # X-Forwarded-For, -By, and -Proto compatability
    ///
    /// This implementation includes fall-back support for the
    /// historical unstandardized headers x-forwarded-for,
    /// x-forwarded-by, and x-forwarded-proto. If you do not wish to
    /// support these headers, use
    /// [`Forwarded::from_forwarded_header`]. To _only_ support these
    /// historical headers and _not_ the standardized Forwarded
    /// header, use [`Forwarded::from_x_headers`].
    ///
    /// Please note that either way, this implementation will
    /// normalize to the standardized Forwarded header, as recommended
    /// in
    /// [rfc7239§7.4](https://tools.ietf.org/html/rfc7239#section-7.4)
    ///
    /// # Examples
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header(
    ///     "Forwarded",
    ///     r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown;proto=https"#
    /// );
    /// let forwarded = Forwarded::from_headers(&request)?.unwrap();
    /// assert_eq!(forwarded.proto(), Some("https"));
    /// assert_eq!(forwarded.forwarded_for(), vec!["192.0.2.43", "[2001:db8:cafe::17]", "unknown"]);
    /// # Ok(()) }
    /// ```
    ///
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header("X-Forwarded-For", "192.0.2.43, 2001:db8:cafe::17, unknown");
    /// request.insert_header("X-Forwarded-Proto", "https");
    /// let forwarded = Forwarded::from_headers(&request)?.unwrap();
    /// assert_eq!(forwarded.forwarded_for(), vec!["192.0.2.43", "[2001:db8:cafe::17]", "unknown"]);
    /// assert_eq!(forwarded.proto(), Some("https"));
    /// assert_eq!(
    ///     forwarded.value()?,
    ///     r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown;proto=https"#
    /// );
    /// # Ok(()) }
    /// ```

    pub fn from_headers(headers: &'a impl AsRef<Headers>) -> Result<Option<Self>, ParseError> {
        if let Some(forwarded) = Self::from_forwarded_header(headers)? {
            Ok(Some(forwarded))
        } else {
            Self::from_x_headers(headers)
        }
    }

    /// Parse a borrowed Forwarded from the Forwarded header, without x-forwarded-{for,by,proto} fallback
    ///
    /// # Examples
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header(
    ///     "Forwarded",
    ///     r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown;proto=https"#
    /// );
    /// let forwarded = Forwarded::from_forwarded_header(&request)?.unwrap();
    /// assert_eq!(forwarded.proto(), Some("https"));
    /// assert_eq!(forwarded.forwarded_for(), vec!["192.0.2.43", "[2001:db8:cafe::17]", "unknown"]);
    /// # Ok(()) }
    /// ```
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header("X-Forwarded-For", "192.0.2.43, 2001:db8:cafe::17");
    /// assert!(Forwarded::from_forwarded_header(&request)?.is_none());
    /// # Ok(()) }
    /// ```
    pub fn from_forwarded_header(
        headers: &'a impl AsRef<Headers>,
    ) -> Result<Option<Self>, ParseError> {
        if let Some(headers) = headers.as_ref().get(FORWARDED) {
            Ok(Some(Self::parse(headers.as_ref().as_str())?))
        } else {
            Ok(None)
        }
    }

    /// Parse a borrowed Forwarded from the historical
    /// non-standardized x-forwarded-{for,by,proto} headers, without
    /// support for the Forwarded header.
    ///
    /// # Examples
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header("X-Forwarded-For", "192.0.2.43, 2001:db8:cafe::17");
    /// let forwarded = Forwarded::from_headers(&request)?.unwrap();
    /// assert_eq!(forwarded.forwarded_for(), vec!["192.0.2.43", "[2001:db8:cafe::17]"]);
    /// # Ok(()) }
    /// ```
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let mut request = Request::new(Get, Url::parse("http://_/")?);
    /// request.insert_header(
    ///     "Forwarded",
    ///     r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown;proto=https"#
    /// );
    /// assert!(Forwarded::from_x_headers(&request)?.is_none());
    /// # Ok(()) }
    /// ```
    pub fn from_x_headers(headers: &'a impl AsRef<Headers>) -> Result<Option<Self>, ParseError> {
        let headers = headers.as_ref();

        let forwarded_for: Vec<Cow<'a, str>> = headers
            .get(X_FORWARDED_FOR)
            .map(|hv| {
                hv.as_str()
                    .split(',')
                    .map(|v| {
                        let v = v.trim();
                        match v.parse::<IpAddr>().ok() {
                            Some(IpAddr::V6(v6)) => Cow::Owned(format!(r#"[{}]"#, v6)),
                            _ => Cow::Borrowed(v),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let by = headers
            .get(X_FORWARDED_BY)
            .map(|hv| Cow::Borrowed(hv.as_str()));

        let proto = headers
            .get(X_FORWARDED_PROTO)
            .map(|p| Cow::Borrowed(p.as_str()));

        if !forwarded_for.is_empty() || by.is_some() || proto.is_some() {
            Ok(Some(Self {
                forwarded_for,
                by,
                proto,
                host: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// parse a &str into a borrowed Forwarded
    ///
    /// # Examples
    /// ```rust
    /// # use http_types::{proxies::Forwarded, Method::Get, Request, Url, Result};
    /// # fn main() -> Result<()> {
    /// let forwarded = Forwarded::parse(
    ///     r#"for=192.0.2.43,         for="[2001:db8:cafe::17]", FOR=unknown;proto=https"#
    /// )?;
    /// assert_eq!(forwarded.forwarded_for(), vec!["192.0.2.43", "[2001:db8:cafe::17]", "unknown"]);
    /// assert_eq!(
    ///     forwarded.value()?,
    ///     r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown;proto=https"#
    /// );
    /// # Ok(()) }
    /// ```
    pub fn parse(input: &'a str) -> Result<Self, ParseError> {
        let mut input = input;
        let mut forwarded = Forwarded::new();

        while !input.is_empty() {
            input = if starts_with_ignore_case("for=", input) {
                forwarded.parse_for(input)?
            } else {
                forwarded.parse_forwarded_pair(input)?
            }
        }

        Ok(forwarded)
    }

    fn parse_forwarded_pair(&mut self, input: &'a str) -> Result<&'a str, ParseError> {
        let (key, value, rest) = match parse_token(input) {
            (Some(key), rest) if rest.starts_with('=') => match parse_value(&rest[1..]) {
                (Some(value), rest) => Some((key, value, rest)),
                (None, _) => None,
            },
            _ => None,
        }
        .ok_or_else(|| ParseError::new("parse error in forwarded-pair"))?;

        match key {
            "by" => {
                if self.by.is_some() {
                    return Err(ParseError::new("parse error, duplicate `by` key"));
                }
                self.by = Some(value);
            }

            "host" => {
                if self.host.is_some() {
                    return Err(ParseError::new("parse error, duplicate `host` key"));
                }
                self.host = Some(value);
            }

            "proto" => {
                if self.proto.is_some() {
                    return Err(ParseError::new("parse error, duplicate `proto` key"));
                }
                self.proto = Some(value);
            }

            _ => { /* extensions are allowed in the spec */ }
        }

        match rest.strip_prefix(';') {
            Some(rest) => Ok(rest),
            None => Ok(rest),
        }
    }

    fn parse_for(&mut self, input: &'a str) -> Result<&'a str, ParseError> {
        let mut rest = input;

        loop {
            rest = match match_ignore_case("for=", rest) {
                (true, rest) => rest,
                (false, _) => return Err(ParseError::new("http list must start with for=")),
            };

            let (value, rest_) = parse_value(rest);
            rest = rest_;

            if let Some(value) = value {
                // add a successful for= value
                self.forwarded_for.push(value);
            } else {
                return Err(ParseError::new("for= without valid value"));
            }

            match rest.chars().next() {
                // we have another for=
                Some(',') => {
                    rest = rest[1..].trim_start();
                }

                // we have reached the end of the for= section
                Some(';') => return Ok(&rest[1..]),

                // reached the end of the input
                None => return Ok(rest),

                // bail
                _ => return Err(ParseError::new("unexpected character after for= section")),
            }
        }
    }

    /// Transform a borrowed Forwarded into an owned
    /// Forwarded. This is a noop if the Forwarded is already owned.
    pub fn into_owned(self) -> Forwarded<'static> {
        Forwarded {
            by: self.by.map(|by| Cow::Owned(by.into_owned())),
            forwarded_for: self
                .forwarded_for
                .into_iter()
                .map(|ff| Cow::Owned(ff.into_owned()))
                .collect(),
            host: self.host.map(|h| Cow::Owned(h.into_owned())),
            proto: self.proto.map(|p| Cow::Owned(p.into_owned())),
        }
    }

    /// Insert a header that represents this Forwarded.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut response = http_types::Response::new(200);
    /// let mut forwarded = http_types::proxies::Forwarded::new();
    /// forwarded.add_for("192.0.2.43");
    /// forwarded.add_for("[2001:db8:cafe::17]");
    /// forwarded.set_proto("https");
    /// forwarded.apply(&mut response);
    /// assert_eq!(response["Forwarded"], r#"for=192.0.2.43, for="[2001:db8:cafe::17]";proto=https"#);
    /// ```
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(FORWARDED, self);
    }

    /// Builds a Forwarded header as a String.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> http_types::Result<()> {
    /// let mut forwarded = http_types::proxies::Forwarded::new();
    /// forwarded.add_for("_haproxy");
    /// forwarded.add_for("[2001:db8:cafe::17]");
    /// forwarded.set_proto("https");
    /// assert_eq!(forwarded.value()?, r#"for=_haproxy, for="[2001:db8:cafe::17]";proto=https"#);
    /// # Ok(()) }
    /// ```
    pub fn value(&self) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();
        if let Some(by) = self.by() {
            write!(&mut buf, "by={};", by)?;
        }

        buf.push_str(
            &self
                .forwarded_for
                .iter()
                .map(|f| format!("for={}", format_value(f)))
                .collect::<Vec<_>>()
                .join(", "),
        );

        buf.push(';');

        if let Some(host) = self.host() {
            write!(&mut buf, "host={};", host)?;
        }

        if let Some(proto) = self.proto() {
            write!(&mut buf, "proto={};", proto)?;
        }

        // remove a trailing semicolon
        buf.pop();

        Ok(buf)
    }

    /// Builds a new empty Forwarded
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a `for` section to this header
    pub fn add_for(&mut self, forwarded_for: impl Into<Cow<'a, str>>) {
        self.forwarded_for.push(forwarded_for.into());
    }

    /// Returns the `for` field of this header
    pub fn forwarded_for(&self) -> Vec<&str> {
        self.forwarded_for.iter().map(|x| x.as_ref()).collect()
    }

    /// Sets the `host` field of this header
    pub fn set_host(&mut self, host: impl Into<Cow<'a, str>>) {
        self.host = Some(host.into());
    }

    /// Returns the `host` field of this header
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Sets the `proto` field of this header
    pub fn set_proto(&mut self, proto: impl Into<Cow<'a, str>>) {
        self.proto = Some(proto.into())
    }

    /// Returns the `proto` field of this header
    pub fn proto(&self) -> Option<&str> {
        self.proto.as_deref()
    }

    /// Sets the `by` field of this header
    pub fn set_by(&mut self, by: impl Into<Cow<'a, str>>) {
        self.by = Some(by.into());
    }

    /// Returns the `by` field of this header
    pub fn by(&self) -> Option<&str> {
        self.by.as_deref()
    }
}

fn parse_value(input: &str) -> (Option<Cow<'_, str>>, &str) {
    match parse_token(input) {
        (Some(token), rest) => (Some(Cow::Borrowed(token)), rest),
        (None, rest) => parse_quoted_string(rest),
    }
}

fn format_value(input: &str) -> Cow<'_, str> {
    match parse_token(input) {
        (_, "") => input.into(),
        _ => {
            let mut string = String::from("\"");
            for ch in input.chars() {
                if let '\\' | '"' = ch {
                    string.push('\\');
                }
                string.push(ch);
            }
            string.push('"');
            string.into()
        }
    }
}

fn match_ignore_case<'a>(start: &'static str, input: &'a str) -> (bool, &'a str) {
    let len = start.len();
    if input[..len].eq_ignore_ascii_case(start) {
        (true, &input[len..])
    } else {
        (false, input)
    }
}

fn starts_with_ignore_case(start: &'static str, input: &str) -> bool {
    if start.len() <= input.len() {
        let len = start.len();
        input[..len].eq_ignore_ascii_case(start)
    } else {
        false
    }
}

impl std::fmt::Display for Forwarded<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value()?)
    }
}

impl ToHeaderValues for Forwarded<'_> {
    type Iter = std::option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        self.value()?.to_header_values()
    }
}

impl ToHeaderValues for &Forwarded<'_> {
    type Iter = std::option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        self.value()?.to_header_values()
    }
}

#[derive(Debug, Clone)]
pub struct ParseError(&'static str);
impl ParseError {
    pub fn new(msg: &'static str) -> Self {
        Self(msg)
    }
}

impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse forwarded header: {}", self.0)
    }
}

impl<'a> TryFrom<&'a str> for Forwarded<'a> {
    type Error = ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Method::Get, Request, Response, Result};
    use url::Url;

    #[test]
    fn starts_with_ignore_case_can_handle_short_inputs() {
        assert!(!starts_with_ignore_case("helloooooo", "h"));
    }

    #[test]
    fn parsing_for() -> Result<()> {
        assert_eq!(
            Forwarded::parse(r#"for="_gazonk""#)?.forwarded_for(),
            vec!["_gazonk"]
        );
        assert_eq!(
            Forwarded::parse(r#"For="[2001:db8:cafe::17]:4711""#)?.forwarded_for(),
            vec!["[2001:db8:cafe::17]:4711"]
        );

        assert_eq!(
            Forwarded::parse("for=192.0.2.60;proto=http;by=203.0.113.43")?.forwarded_for(),
            vec!["192.0.2.60"]
        );

        assert_eq!(
            Forwarded::parse("for=192.0.2.43,   for=198.51.100.17")?.forwarded_for(),
            vec!["192.0.2.43", "198.51.100.17"]
        );

        assert_eq!(
            Forwarded::parse(r#"for=192.0.2.43,for="[2001:db8:cafe::17]",for=unknown"#)?
                .forwarded_for(),
            Forwarded::parse(r#"for=192.0.2.43, for="[2001:db8:cafe::17]", for=unknown"#)?
                .forwarded_for()
        );

        assert_eq!(
            Forwarded::parse(
                r#"for=192.0.2.43,for="this is a valid quoted-string, \" \\",for=unknown"#
            )?
            .forwarded_for(),
            vec![
                "192.0.2.43",
                r#"this is a valid quoted-string, " \"#,
                "unknown"
            ]
        );

        Ok(())
    }

    #[test]
    fn basic_parse() -> Result<()> {
        let forwarded = Forwarded::parse("for=client.com;by=proxy.com;host=host.com;proto=https")?;

        assert_eq!(forwarded.by(), Some("proxy.com"));
        assert_eq!(forwarded.forwarded_for(), vec!["client.com"]);
        assert_eq!(forwarded.host(), Some("host.com"));
        assert_eq!(forwarded.proto(), Some("https"));
        assert!(matches!(forwarded, Forwarded { .. }));
        Ok(())
    }

    #[test]
    fn bad_parse() {
        let err = Forwarded::parse("by=proxy.com;for=client;host=example.com;host").unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse forwarded header: parse error in forwarded-pair"
        );

        let err = Forwarded::parse("by;for;host;proto").unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse forwarded header: parse error in forwarded-pair"
        );

        let err = Forwarded::parse("for=for, key=value").unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse forwarded header: http list must start with for="
        );

        let err = Forwarded::parse(r#"for="unterminated string"#).unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse forwarded header: for= without valid value"
        );

        let err = Forwarded::parse(r#"for=, for=;"#).unwrap_err();
        assert_eq!(
            err.to_string(),
            "unable to parse forwarded header: for= without valid value"
        );
    }

    #[test]
    fn bad_parse_from_headers() -> Result<()> {
        let mut response = Response::new(200);
        response.append_header("forwarded", "uh oh");
        assert_eq!(
            Forwarded::from_headers(&response).unwrap_err().to_string(),
            "unable to parse forwarded header: parse error in forwarded-pair"
        );

        let response = Response::new(200);
        assert!(Forwarded::from_headers(&response)?.is_none());
        Ok(())
    }

    #[test]
    fn from_x_headers() -> Result<()> {
        let mut request = Request::new(Get, Url::parse("http://_/")?);
        request.append_header(X_FORWARDED_FOR, "192.0.2.43, 2001:db8:cafe::17");
        request.append_header(X_FORWARDED_PROTO, "gopher");
        let forwarded = Forwarded::from_headers(&request)?.unwrap();
        assert_eq!(
            forwarded.to_string(),
            r#"for=192.0.2.43, for="[2001:db8:cafe::17]";proto=gopher"#
        );
        Ok(())
    }

    #[test]
    fn formatting_edge_cases() {
        let mut forwarded = Forwarded::new();
        forwarded.add_for(r#"quote: " backslash: \"#);
        forwarded.add_for(";proto=https");
        assert_eq!(
            forwarded.to_string(),
            r#"for="quote: \" backslash: \\", for=";proto=https""#
        );
    }

    #[test]
    fn parse_edge_cases() -> Result<()> {
        let forwarded =
            Forwarded::parse(r#"for=";", for=",", for="\"", for=unquoted;by=";proto=https""#)?;
        assert_eq!(forwarded.forwarded_for(), vec![";", ",", "\"", "unquoted"]);
        assert_eq!(forwarded.by(), Some(";proto=https"));
        assert!(forwarded.proto().is_none());

        let forwarded = Forwarded::parse("proto=https")?;
        assert_eq!(forwarded.proto(), Some("https"));
        Ok(())
    }

    #[test]
    fn owned_parse() -> Result<()> {
        let forwarded =
            Forwarded::parse("for=client;by=proxy.com;host=example.com;proto=https")?.into_owned();

        assert_eq!(forwarded.by(), Some("proxy.com"));
        assert_eq!(forwarded.forwarded_for(), vec!["client"]);
        assert_eq!(forwarded.host(), Some("example.com"));
        assert_eq!(forwarded.proto(), Some("https"));
        assert!(matches!(forwarded, Forwarded { .. }));
        Ok(())
    }

    #[test]
    fn from_request() -> Result<()> {
        let mut request = Request::new(Get, Url::parse("http://_/")?);
        request.append_header("Forwarded", "for=for");

        let forwarded = Forwarded::from_headers(&request)?.unwrap();
        assert_eq!(forwarded.forwarded_for(), vec!["for"]);

        Ok(())
    }

    #[test]
    fn owned_can_outlive_request() -> Result<()> {
        let forwarded = {
            let mut request = Request::new(Get, Url::parse("http://_/")?);
            request.append_header("Forwarded", "for=for;by=by;host=host;proto=proto");
            Forwarded::from_headers(&request)?.unwrap().into_owned()
        };
        assert_eq!(forwarded.by(), Some("by"));
        Ok(())
    }

    #[test]
    fn round_trip() -> Result<()> {
        let inputs = [
            "for=client,for=b,for=c;by=proxy.com;host=example.com;proto=https",
            "by=proxy.com;proto=https;host=example.com;for=a,for=b",
        ];
        for input in inputs {
            let forwarded = Forwarded::parse(input).map_err(|_| crate::Error::new_adhoc(input))?;
            let header = forwarded.to_header_values()?.next().unwrap();
            let parsed = Forwarded::parse(header.as_str())?;
            assert_eq!(forwarded, parsed);
        }
        Ok(())
    }
}
