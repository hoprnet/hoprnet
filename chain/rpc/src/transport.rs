/// Extend implementation of `alloy::transport::Http` so that it can
/// operate with [`surf`]. This should be used in `async-std` runtime
use alloy::{
    rpc::json_rpc::{RequestPacket, ResponsePacket},
    transports::{
        http::HttpConnect, utils::guess_local_url, BoxTransport, TransportConnect, TransportError, TransportErrorKind,
        TransportFut, TransportResult,
    },
};

use std::task;
pub use surf::Client as SurfClient;
use tower::Service;
use tracing::{debug, debug_span, info, trace, Instrument};
use url::Url;

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

// impl<T> From<Http<T>> for HttpWrapper<T> {
//     fn from(value: Http<T>) -> Self {
//         Self {
//             client: *value.client(),
//             url: Url::parse(value.url()).unwrap(),
//         }
//     }
// }

// impl<T> From<HttpWrapper<T>> for Http<T> {
//     fn from(value: HttpWrapper<T>) -> Self {
//         Self::with_client(*value.client(), Url::parse(value.url()).unwrap())
//     }
// }

/// An [`Http`] transport using [`surf`].
// pub struct SurfTransport(Http<SurfClient>);
pub type SurfTransport = HttpWrapper<SurfClient>;

impl SurfTransport {
    /// Create a new [`SurfTransport`] with the given URL and default client.
    pub fn new(url: Url) -> Self {
        info!("creating surf client");
        Self {
            client: surf::client(),
            url,
        }
    }

    /// Do POST query with SurfRequestor
    async fn do_surf(self, req: RequestPacket) -> TransportResult<ResponsePacket> {
        // when POST method
        let req = self
            .client
            .post(self.url)
            .content_type("application/json")
            .body_json(&req)
            .map_err(|e| TransportErrorKind::Custom(e.to_string().into()))?;

        let mut res = req
            .await
            .map_err(|e| TransportErrorKind::Custom(e.to_string().into()))?;

        let status = res.status();

        debug!(%status, "received response from server");

        let body = res
            .body_bytes()
            .await
            .map_err(|e| TransportErrorKind::Custom(e.to_string().into()))?;

        debug!(bytes = body.len(), "retrieved response body. Use `trace` for full body");
        trace!(body = %String::from_utf8_lossy(&body), "response body");

        if !status.is_success() {
            return Err(TransportErrorKind::http_error(
                status as u16,
                String::from_utf8_lossy(&body).into_owned(),
            ));
        }

        // Deserialize a Box<RawValue> from the body. If deserialization fails, return
        // the body as a string in the error. The conversion to String
        // is lossy and may not cover all the bytes in the body.
        serde_json::from_slice(&body)
            .map_err(|err| TransportError::deser_err(err, String::from_utf8_lossy(body.as_ref())))
    }
}

/// Connection details for a [`SurfTransport`].
pub type SurfConnect = HttpConnect<SurfTransport>;

impl TransportConnect for SurfTransport {
    fn is_local(&self) -> bool {
        guess_local_url(self.url.as_str())
    }

    async fn get_transport(&self) -> Result<BoxTransport, TransportError> {
        Ok(BoxTransport::new(HttpWrapper::with_client(
            SurfClient::new(),
            self.url.clone(),
        )))
    }
}

impl Service<RequestPacket> for HttpWrapper<SurfClient> {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // always returns `Ok(())`.
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let this = self.clone();
        let span = debug_span!("SurfTransport", url = %this.url);
        Box::pin(this.do_surf(req).instrument(span))
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::SurfTransport;
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

        let num = provider.get_block_number().await.unwrap();

        assert_eq!(num, 0);

        Ok(())
    }
}
