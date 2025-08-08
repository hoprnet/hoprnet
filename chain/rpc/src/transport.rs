use alloy::transports::{http::Http, utils::guess_local_url};
use async_trait::async_trait;
#[cfg(feature = "runtime-tokio")]
pub use reqwest::Client as ReqwestClient;
use url::Url;

use crate::errors::HttpRequestError;

/// Abstraction for an HTTP client that performs HTTP GET with serializable request data.
#[async_trait]
pub trait HttpRequestor: std::fmt::Debug + Send + Sync {
    /// Performs HTTP GET query to the given URL and gets the JSON response.
    async fn http_get(&self, url: &str) -> std::result::Result<Box<[u8]>, HttpRequestError>;
}

/// Local wrapper for `Http`
#[derive(Clone, Debug)]
pub struct HttpWrapper<T> {
    client: T,
    url: Url,
}

impl<T> HttpWrapper<T> {
    /// Create a new [`Http`] transport with a custom client.
    pub const fn with_client(client: T, url: Url) -> Self {
        Self { client, url }
    }

    /// Set the URL.
    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }

    /// Set the client.
    pub fn set_client(&mut self, client: T) {
        self.client = client;
    }

    /// Guess whether the URL is local, based on the hostname.
    ///
    /// The output of this function is best-efforts, and should be checked if
    /// possible. It simply returns `true` if the connection has no hostname,
    /// or the hostname is `localhost` or `127.0.0.1`.
    pub fn guess_local(&self) -> bool {
        guess_local_url(&self.url)
    }

    /// Get a reference to the client.
    pub const fn client(&self) -> &T {
        &self.client
    }

    /// Get a reference to the URL.
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
}

impl<T: Clone> From<Http<T>> for HttpWrapper<T> {
    fn from(value: Http<T>) -> Self {
        Self {
            client: value.client().clone(),
            url: Url::parse(value.url()).unwrap(),
        }
    }
}

impl<T: Clone> From<HttpWrapper<T>> for Http<T> {
    fn from(value: HttpWrapper<T>) -> Self {
        Self::with_client(value.client().clone(), Url::parse(value.url()).unwrap())
    }
}

#[cfg(feature = "runtime-tokio")]
#[async_trait]
impl HttpRequestor for ReqwestClient {
    #[inline]
    async fn http_get(&self, url: &str) -> std::result::Result<Box<[u8]>, HttpRequestError> {
        let res = self
            .get(url)
            .send()
            .await
            .map_err(|e| HttpRequestError::TransportError(e.to_string()))?;

        let status = res.status();

        tracing::debug!(%status, "received response from server");

        let body = res
            .bytes()
            .await
            .map_err(|e| HttpRequestError::UnknownError(e.to_string()))?;

        tracing::debug!(bytes = body.len(), "retrieved response body. Use `trace` for full body");
        tracing::trace!(body = %String::from_utf8_lossy(&body), "response body");

        if !status.is_success() {
            return Err(HttpRequestError::HttpError(
                http::StatusCode::try_from(status.as_u16()).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
            ));
        }

        Ok(body.to_vec().into_boxed_slice())
    }
}
