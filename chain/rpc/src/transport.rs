use alloy::transports::{http::Http, utils::guess_local_url};
use async_trait::async_trait;
pub use reqwest::Client as ReqwestClient;
use tracing::{debug, trace};
use url::Url;

use crate::errors::HttpRequestError;

/// Abstraction for an HTTP client that performs HTTP GET with serializable request data.
#[async_trait]
pub trait HttpRequestor: std::fmt::Debug + Send + Sync + Clone {
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

        debug!(%status, "received response from server");

        let body = res
            .bytes()
            .await
            .map_err(|e| HttpRequestError::UnknownError(e.to_string().into()))?;

        debug!(bytes = body.len(), "retrieved response body. Use `trace` for full body");
        trace!(body = %String::from_utf8_lossy(&body), "response body");

        if !status.is_success() {
            return Err(HttpRequestError::HttpError(
                http_types::StatusCode::try_from(status.as_u16())
                    .unwrap_or(http_types::StatusCode::InternalServerError),
            ));
        }

        Ok(body.to_vec().into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::{HttpRequestor, SurfTransport};
    use alloy::{
        providers::{Provider, ProviderBuilder},
        rpc::client::ClientBuilder,
    };
    use hopr_chain_types::utils::create_anvil;

    #[tokio::test]
    async fn test_surf_transport_tokio() {
        let _ = test_surf_transport().await;
    }

    #[async_std::test]
    async fn test_surf_transport_std() {
        let _ = test_surf_transport().await;
    }

    async fn test_surf_transport() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(None);

        let surf_transport_client = SurfTransport::new(anvil.endpoint_url());
        let rpc_client =
            ClientBuilder::default().transport(surf_transport_client.clone(), surf_transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let num = provider.get_block_number().await?;

        assert_eq!(num, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_surf_transport_get() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("GET", "/gasapi.ashx?apikey=key&method=gasoracle")
            .with_status(http_types::StatusCode::Accepted as usize)
            .with_body(r#"{"status":"1","message":"OK","result":{"LastBlock":"39864926","SafeGasPrice":"1.1","ProposeGasPrice":"1.1","FastGasPrice":"1.6","UsdPrice":"0.999968207972734"}}"#)
            .expect(1)
            .create();

        let anvil = create_anvil(None);

        let surf_transport_client = SurfTransport::new(anvil.endpoint_url());

        let url = server.url();
        let req_uri = format!("{}/gasapi.ashx?apikey=key&method=gasoracle", url);
        let resp = surf_transport_client.client().http_get(&req_uri).await?;

        assert_eq!(
            resp.iter().as_slice(),
            r#"{"status":"1","message":"OK","result":{"LastBlock":"39864926","SafeGasPrice":"1.1","ProposeGasPrice":"1.1","FastGasPrice":"1.6","UsdPrice":"0.999968207972734"}}"#.as_bytes()
        );

        Ok(())
    }
}
