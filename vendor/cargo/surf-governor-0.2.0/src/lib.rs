#![forbid(unsafe_code, future_incompatible)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unused_qualifications,
    unused_import_braces,
    unused_extern_crates,
    trivial_casts,
    trivial_numeric_casts
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! A [surf] middleware that implements rate-limiting using [governor].
//! The majority of this has been copied from [tide-governor](https://github.com/ohmree/tide-governor)
//! # Example
//! ```no_run
//! use surf_governor::GovernorMiddleware;
//! use surf::{Client, Request, http::Method};
//! use url::Url;
//!
//! #[async_std::main]
//! async fn main() -> surf::Result<()> {
//!     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
//!     // Construct Surf client with a governor
//!     let client = Client::new().with(GovernorMiddleware::per_second(30)?);
//!     let res = client.send(req).await?;
//!     Ok(())
//! }
//! ```
//! [surf]: https://github.com/http-rs/surf
//! [governor]: https://github.com/antifuchs/governor

// TODO: figure out how to add jitter support using `governor::Jitter`.
// TODO: add more unit tests.
use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use http_types::{headers, Response, StatusCode};
use lazy_static::lazy_static;
use std::{convert::TryInto, error::Error, num::NonZeroU32, sync::Arc, time::Duration};
use surf::{middleware::Next, Client, Request, Result};

lazy_static! {
    static ref CLOCK: DefaultClock = DefaultClock::default();
}

/// Once the rate limit has been reached, the middleware will respond with
/// status code 429 (too many requests) and a `Retry-After` header with the amount
/// of time that needs to pass before another request will be allowed.
#[derive(Debug, Clone)]
pub struct GovernorMiddleware {
    limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

impl GovernorMiddleware {
    /// Constructs a rate-limiting middleware from a [`Duration`] that allows one request in the given time interval.
    ///
    /// If the time interval is zero, returns `None`.
    /// # Example
    /// This constructs a client with a governor set to 1 requests every 5 nanoseconds
    /// ```no_run
    /// use surf_governor::GovernorMiddleware;
    /// use surf::{Client, Request, http::Method};
    /// use url::Url;
    ///
    /// use std::time::Duration;
    ///
    /// #[async_std::main]
    /// async fn main() -> surf::Result<()> {
    ///     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
    ///     // Construct Surf client with a governor
    ///     let client = Client::new().with(GovernorMiddleware::with_period(Duration::new(0, 5)).unwrap());
    ///     let res = client.send(req).await?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn with_period(duration: Duration) -> Option<Self> {
        Some(Self {
            limiter: Arc::new(RateLimiter::<String, _, _>::keyed(Quota::with_period(
                duration,
            )?)),
        })
    }

    /// Constructs a rate-limiting middleware that allows a specified number of requests every second.
    ///
    /// Returns an error if `times` can't be converted into a [`NonZeroU32`].
    ///
    /// # Example
    /// This constructs a client with a governor set to 30 requests per second limit
    /// ```no_run
    /// use surf_governor::GovernorMiddleware;
    /// use surf::{Client, Request, http::Method};
    /// use url::Url;
    ///
    /// #[async_std::main]
    /// async fn main() -> surf::Result<()> {
    ///     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
    ///     // Construct Surf client with a governor
    ///     let client = Client::new().with(GovernorMiddleware::per_second(30)?);
    ///     let res = client.send(req).await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn per_second<T>(times: T) -> Result<Self>
    where
        T: TryInto<NonZeroU32>,
        T::Error: Error + Send + Sync + 'static,
    {
        Ok(Self {
            limiter: Arc::new(RateLimiter::<String, _, _>::keyed(Quota::per_second(
                times.try_into()?,
            ))),
        })
    }

    /// Constructs a rate-limiting middleware that allows a specified number of requests every minute.
    ///
    /// Returns an error if `times` can't be converted into a [`NonZeroU32`].
    ///
    /// # Example
    /// This constructs a client with a governor set to 300 requests per minute limit
    /// ```no_run
    /// use surf_governor::GovernorMiddleware;
    /// use surf::{Client, Request, http::Method};
    /// use url::Url;
    ///
    /// #[async_std::main]
    /// async fn main() -> surf::Result<()> {
    ///     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
    ///     // Construct Surf client with a governor
    ///     let client = Client::new().with(GovernorMiddleware::per_minute(300)?);
    ///     let res = client.send(req).await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn per_minute<T>(times: T) -> Result<Self>
    where
        T: TryInto<NonZeroU32>,
        T::Error: Error + Send + Sync + 'static,
    {
        Ok(Self {
            limiter: Arc::new(RateLimiter::<String, _, _>::keyed(Quota::per_minute(
                times.try_into()?,
            ))),
        })
    }

    /// Constructs a rate-limiting middleware that allows a specified number of requests every hour.
    ///
    /// Returns an error if `times` can't be converted into a [`NonZeroU32`].
    ///
    /// # Example
    /// This constructs a client with a governor set to 3000 requests per hour limit
    /// ```no_run
    /// use surf_governor::GovernorMiddleware;
    /// use surf::{Client, Request, http::Method};
    /// use url::Url;
    ///
    /// #[async_std::main]
    /// async fn main() -> surf::Result<()> {
    ///     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
    ///     // Construct Surf client with a governor
    ///     let client = Client::new().with(GovernorMiddleware::per_hour(3000)?);
    ///     let res = client.send(req).await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn per_hour<T>(times: T) -> Result<Self>
    where
        T: TryInto<NonZeroU32>,
        T::Error: Error + Send + Sync + 'static,
    {
        Ok(Self {
            limiter: Arc::new(RateLimiter::<String, _, _>::keyed(Quota::per_hour(
                times.try_into()?,
            ))),
        })
    }
}

#[surf::utils::async_trait]
impl surf::middleware::Middleware for GovernorMiddleware {
    async fn handle(
        &self,
        req: Request,
        client: Client,
        next: Next<'_>,
    ) -> std::result::Result<surf::Response, http_types::Error> {
        match self
            .limiter
            .check_key(&req.url().host_str().unwrap().to_string())
        {
            Ok(_) => Ok(next.run(req, client).await?),
            Err(negative) => {
                let wait_time = negative.wait_time_from(CLOCK.now());
                let mut res = Response::new(StatusCode::TooManyRequests);
                res.insert_header(headers::RETRY_AFTER, wait_time.as_secs().to_string());
                Ok(res.try_into()?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::GovernorMiddleware;
    use surf::{http::Method, Client, Request};
    use url::Url;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};
    #[async_std::test]
    async fn limits_requests() -> surf::Result<()> {
        let mock_server = MockServer::start().await;
        let m = Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello!".to_string()))
            .expect(1);
        let _mock_guard = mock_server.register_as_scoped(m).await;
        let url = format!("{}/", &mock_server.uri());
        let req = Request::new(Method::Get, Url::parse(&url).unwrap());
        let client = Client::new().with(GovernorMiddleware::per_second(1)?);
        let good_res = client.send(req.clone()).await?;
        assert_eq!(good_res.status(), 200);
        let wait_res = client.send(req).await?;
        assert_eq!(wait_res.status(), 429);
        Ok(())
    }
}
