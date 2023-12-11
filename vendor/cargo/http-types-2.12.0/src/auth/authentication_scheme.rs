use std::fmt::{self, Display};
use std::str::FromStr;

use crate::bail_status as bail;

/// HTTP Mutual Authentication Algorithms
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum AuthenticationScheme {
    /// [RFC7617](https://tools.ietf.org/html/rfc7617) Basic auth
    Basic,
    /// [RFC6750](https://tools.ietf.org/html/rfc6750) Bearer auth
    Bearer,
    /// [RFC7616](https://tools.ietf.org/html/rfc7616) Digest auth
    Digest,
    /// [RFC7486](https://tools.ietf.org/html/rfc7486) HTTP Origin-Bound Authentication (HOBA)
    Hoba,
    /// [RFC8120](https://tools.ietf.org/html/rfc8120) Mutual auth
    Mutual,
    /// [RFC4559](https://tools.ietf.org/html/rfc4559) Negotiate auth
    Negotiate,
    /// [RFC5849](https://tools.ietf.org/html/rfc5849) OAuth
    OAuth,
    /// [RFC7804](https://tools.ietf.org/html/rfc7804) SCRAM SHA1 auth
    ScramSha1,
    /// [RFC7804](https://tools.ietf.org/html/rfc7804) SCRAM SHA256 auth
    ScramSha256,
    /// [RFC8292](https://tools.ietf.org/html/rfc8292) Vapid auth
    Vapid,
}

impl Display for AuthenticationScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic => write!(f, "Basic"),
            Self::Bearer => write!(f, "Bearer"),
            Self::Digest => write!(f, "Digest"),
            Self::Hoba => write!(f, "HOBA"),
            Self::Mutual => write!(f, "Mutual"),
            Self::Negotiate => write!(f, "Negotiate"),
            Self::OAuth => write!(f, "OAuth"),
            Self::ScramSha1 => write!(f, "SCRAM-SHA-1"),
            Self::ScramSha256 => write!(f, "SCRAM-SHA-256"),
            Self::Vapid => write!(f, "vapid"),
        }
    }
}

impl FromStr for AuthenticationScheme {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE(yosh): matching here is lowercase as specified by RFC2617#section-1.2
        // > [...] case-insensitive token to identify the authentication scheme [...]
        // https://tools.ietf.org/html/rfc2617#section-1.2
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "bearer" => Ok(Self::Bearer),
            "digest" => Ok(Self::Digest),
            "hoba" => Ok(Self::Hoba),
            "mutual" => Ok(Self::Mutual),
            "negotiate" => Ok(Self::Negotiate),
            "oauth" => Ok(Self::OAuth),
            "scram-sha-1" => Ok(Self::ScramSha1),
            "scram-sha-256" => Ok(Self::ScramSha256),
            "vapid" => Ok(Self::Vapid),
            s => bail!(400, "`{}` is not a recognized authentication scheme", s),
        }
    }
}
