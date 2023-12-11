use crate::headers::HeaderValue;
use crate::Status;

use std::time::Duration;

/// An HTTP `Cache-Control` directive.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheDirective {
    /// The response body will not change over time.
    Immutable,
    /// The maximum amount of time a resource is considered fresh.
    MaxAge(Duration),
    /// Indicates the client will accept a stale response.
    MaxStale(Option<Duration>),
    /// A response that will still be fresh for at least the specified duration.
    MinFresh(Duration),
    /// Once a response is stale, a fresh response must be retrieved.
    MustRevalidate,
    /// The response may be cached, but must always be revalidated before being used.
    NoCache,
    /// The response may not be cached.
    NoStore,
    /// An intermediate cache or proxy should not edit the response body,
    /// Content-Encoding, Content-Range, or Content-Type.
    NoTransform,
    /// Do not use the network for a response.
    OnlyIfCached,
    /// The response may be stored only by a browser's cache, even if the
    /// response is normally non-cacheable.
    Private,
    /// Like must-revalidate, but only for shared caches (e.g., proxies).
    ProxyRevalidate,
    /// The response may be stored by any cache, even if the response is normally
    /// non-cacheable.
    Public,
    /// Overrides max-age or the Expires header, but only for shared caches.
    SMaxAge(Duration),
    /// The client will accept a stale response if retrieving a fresh one fails.
    StaleIfError(Duration),
    /// Indicates the client will accept a stale response, while asynchronously
    /// checking in the background for a fresh one.
    StaleWhileRevalidate(Duration),
}

impl CacheDirective {
    /// Check whether this directive is valid in an HTTP request.
    pub fn valid_in_req(&self) -> bool {
        use CacheDirective::*;
        matches!(
            self,
            MaxAge(_) | MaxStale(_) | MinFresh(_) | NoCache | NoStore | NoTransform | OnlyIfCached
        )
    }

    /// Check whether this directive is valid in an HTTP response.
    pub fn valid_in_res(&self) -> bool {
        use CacheDirective::*;
        matches!(
            self,
            MustRevalidate
                | NoCache
                | NoStore
                | NoTransform
                | Public
                | Private
                | ProxyRevalidate
                | MaxAge(_)
                | SMaxAge(_)
                | StaleIfError(_)
                | StaleWhileRevalidate(_)
        )
    }

    /// Create an instance from a string slice.
    //
    // This is a private method rather than a trait because we assume the
    // input string is a single-value only. This is upheld by the calling
    // function, but we cannot guarantee this to be true in the general
    // sense.
    pub(crate) fn from_str(s: &str) -> crate::Result<Option<Self>> {
        use CacheDirective::*;

        let s = s.trim();

        // We're dealing with an empty string.
        if s.is_empty() {
            return Ok(None);
        }

        s.to_lowercase();
        let mut parts = s.split('=');
        let next = parts.next().unwrap();

        let mut get_dur = || -> crate::Result<Duration> {
            let dur = parts.next().status(400)?;
            let dur: u64 = dur.parse().status(400)?;
            Ok(Duration::new(dur, 0))
        };

        // This won't panic because each input string has at least one part.
        let res = match next {
            "immutable" => Some(Immutable),
            "no-cache" => Some(NoCache),
            "no-store" => Some(NoStore),
            "no-transform" => Some(NoTransform),
            "only-if-cached" => Some(OnlyIfCached),
            "must-revalidate" => Some(MustRevalidate),
            "public" => Some(Public),
            "private" => Some(Private),
            "proxy-revalidate" => Some(ProxyRevalidate),
            "max-age" => Some(MaxAge(get_dur()?)),
            "max-stale" => match parts.next() {
                Some(secs) => {
                    let dur: u64 = secs.parse().status(400)?;
                    Some(MaxStale(Some(Duration::new(dur, 0))))
                }
                None => Some(MaxStale(None)),
            },
            "min-fresh" => Some(MinFresh(get_dur()?)),
            "s-maxage" => Some(SMaxAge(get_dur()?)),
            "stale-if-error" => Some(StaleIfError(get_dur()?)),
            "stale-while-revalidate" => Some(StaleWhileRevalidate(get_dur()?)),
            _ => None,
        };
        Ok(res)
    }
}

impl From<CacheDirective> for HeaderValue {
    fn from(directive: CacheDirective) -> Self {
        use CacheDirective::*;
        let h = |s: String| unsafe { HeaderValue::from_bytes_unchecked(s.into_bytes()) };

        match directive {
            Immutable => h("immutable".to_string()),
            MaxAge(dur) => h(format!("max-age={}", dur.as_secs())),
            MaxStale(dur) => match dur {
                Some(dur) => h(format!("max-stale={}", dur.as_secs())),
                None => h("max-stale".to_string()),
            },
            MinFresh(dur) => h(format!("min-fresh={}", dur.as_secs())),
            MustRevalidate => h("must-revalidate".to_string()),
            NoCache => h("no-cache".to_string()),
            NoStore => h("no-store".to_string()),
            NoTransform => h("no-transform".to_string()),
            OnlyIfCached => h("only-if-cached".to_string()),
            Private => h("private".to_string()),
            ProxyRevalidate => h("proxy-revalidate".to_string()),
            Public => h("public".to_string()),
            SMaxAge(dur) => h(format!("s-max-age={}", dur.as_secs())),
            StaleIfError(dur) => h(format!("stale-if-error={}", dur.as_secs())),
            StaleWhileRevalidate(dur) => h(format!("stale-while-revalidate={}", dur.as_secs())),
        }
    }
}
