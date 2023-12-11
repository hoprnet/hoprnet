use crate::headers::HeaderValue;
use std::fmt::{self, Display};

/// Available compression algorithms.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Encoding {
    /// The Gzip encoding.
    Gzip,
    /// The Deflate encoding.
    Deflate,
    /// The Brotli encoding.
    Brotli,
    /// The Zstd encoding.
    Zstd,
    /// No encoding.
    Identity,
}

impl Encoding {
    /// Parses a given string into its corresponding encoding.
    pub(crate) fn from_str(s: &str) -> Option<Encoding> {
        let s = s.trim();

        // We're dealing with an empty string.
        if s.is_empty() {
            return None;
        }

        match s {
            "gzip" => Some(Encoding::Gzip),
            "deflate" => Some(Encoding::Deflate),
            "br" => Some(Encoding::Brotli),
            "zstd" => Some(Encoding::Zstd),
            "identity" => Some(Encoding::Identity),
            _ => None,
        }
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Encoding::Gzip => write!(f, "gzip"),
            Encoding::Deflate => write!(f, "deflate"),
            Encoding::Brotli => write!(f, "br"),
            Encoding::Zstd => write!(f, "zstd"),
            Encoding::Identity => write!(f, "identity"),
        }
    }
}

impl From<Encoding> for HeaderValue {
    fn from(directive: Encoding) -> Self {
        let s = directive.to_string();
        unsafe { HeaderValue::from_bytes_unchecked(s.into_bytes()) }
    }
}
