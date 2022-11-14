use std::net::{Ipv4Addr, Ipv6Addr};
use url::ParseError as UrlError;
use url::Url;

pub trait IntoUrl {
    fn into_url(self) -> Result<Url, UrlError>;
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, UrlError> {
        Ok(self)
    }
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url, UrlError> {
        Url::parse(self)
    }
}

impl<'a> IntoUrl for &'a String {
    fn into_url(self) -> Result<Url, UrlError> {
        Url::parse(self)
    }
}

pub fn is_http_scheme(url: &Url) -> bool {
    url.scheme().starts_with("http")
}

pub fn is_host_name(host: &str) -> bool {
    host.parse::<Ipv4Addr>().is_err() && host.parse::<Ipv6Addr>().is_err()
}

pub fn is_secure(url: &Url) -> bool {
    url.scheme() == "https"
}

#[cfg(test)]
pub mod test {
    use crate::cookie::Cookie;
    use time::{now_utc, Duration, Tm};
    use url::Url;
    #[inline]
    pub fn url(url: &str) -> Url {
        Url::parse(url).unwrap()
    }
    #[inline]
    pub fn make_cookie<'a>(
        cookie: &str,
        url_str: &str,
        expires: Option<Tm>,
        max_age: Option<u64>,
    ) -> Cookie<'a> {
        Cookie::parse(
            format!(
                "{}{}{}",
                cookie,
                expires.map_or(String::from(""), |e| format!("; Expires={}", e.rfc822())),
                max_age.map_or(String::from(""), |m| format!("; Max-Age={}", m))
            ),
            &url(url_str),
        )
        .unwrap()
    }
    #[inline]
    pub fn in_days(days: i64) -> Tm {
        now_utc() + Duration::days(days)
    }
    #[inline]
    pub fn in_minutes(mins: i64) -> Tm {
        now_utc() + Duration::minutes(mins)
    }
}
