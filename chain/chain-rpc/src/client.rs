use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError, RetryPolicy};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::errors::{HttpRequestError, JsonRpcProviderClientError};
use crate::helper::{Request, Response};
use crate::HttpPostRequestor;

#[cfg(feature = "prometheus")]
use metrics::metrics::MultiCounter;

#[cfg(feature = "prometheus")]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_RPC_CALLS: MultiCounter = MultiCounter::new(
        "chain_counter_rpc_calls",
        "Number of successful RPC calls over HTTP",
        &[
            "eth_blockNumber", "eth_getBalance", "eth_sign", "eth_signTransaction", "eth_sendTransaction",
            "eth_sendRawTransaction", "eth_call", "eth_getBlockByHash", "eth_getBlockByNumber",
            "eth_getTransactionByHash", "eth_getLogs"
        ]
    )
    .unwrap();
}

/// Modified implementation of `ethers::providers::Http` so that it can
/// operate with any `HttpPostRequestor`.
#[derive(Debug)]
pub struct JsonRpcProviderClient<Req: HttpPostRequestor + Debug> {
    id: AtomicU64,
    url: String,
    requestor: Req,
}

impl<Req: HttpPostRequestor + Debug> JsonRpcProviderClient<Req> {
    /// Creates the client given the `HttpPostRequestor`
    pub fn new(base_url: &str, requestor: Req) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: base_url.to_owned(),
            requestor,
        }
    }
}

impl<Req: HttpPostRequestor + Debug + Clone> Clone for JsonRpcProviderClient<Req> {
    fn clone(&self) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: self.url.clone(),
            requestor: self.requestor.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Req: HttpPostRequestor + Debug> JsonRpcClient for JsonRpcProviderClient<Req> {
    type Error = JsonRpcProviderClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        // Create and serialize the Request object
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);
        let serialized_payload = serde_json::to_string(&payload).map_err(|err| Self::Error::SerdeJson {
            err,
            text: "cannot serialize payload".into(),
        })?;

        // Perform the actual request
        let body = self.requestor.http_post(self.url.as_ref(), &serialized_payload).await?;

        // First deserialize the Response object
        let raw = match serde_json::from_str(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => return Err(error.into()),
            Ok(_) => {
                let err = Self::Error::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: body,
                };
                return Err(err);
            }
            Err(err) => return Err(Self::Error::SerdeJson { err, text: body }),
        };

        // Next, deserialize the data out of the Response object
        let res = serde_json::from_str(raw.get()).map_err(|err| Self::Error::SerdeJson {
            err,
            text: raw.to_string(),
        })?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment(&[method]);

        Ok(res)
    }
}

/// Retry policy to use with the RPC Provider
#[derive(Debug)]
pub struct SimpleJsonRpcRetryPolicy;

impl SimpleJsonRpcRetryPolicy {
    fn should_retry_on_json_rpc_error(&self, err: &JsonRpcError) -> bool {
        let JsonRpcError { code, message, .. } = err;

        // Alchemy throws it this way
        if *code == 429 {
            return true;
        }

        // This is an Infura error code for `exceeded project rate limit`
        if *code == -32005 {
            return true;
        }

        // Alternative alchemy error for specific IPs
        if *code == -32016 && message.contains("rate limit") {
            return true;
        }

        match message.as_str() {
            // This is commonly thrown by infura and is apparently a load balancer issue, see also <https://github.com/MetaMask/metamask-extension/issues/7234>
            "header not found" => true,
            // also thrown by Infura if out of budget for the day and rate-limited
            "daily request count exceeded, request rate limited" => true,
            _ => false,
        }
    }
}

impl RetryPolicy<JsonRpcProviderClientError> for SimpleJsonRpcRetryPolicy {
    fn should_retry(&self, error: &JsonRpcProviderClientError) -> bool {
        // There are 3 error cases:
        // - serialization errors of request/response
        // - JSON RPC errors
        // - HTTP & transport errors
        match error {
            // Serialization error
            JsonRpcProviderClientError::SerdeJson { text, .. } => {
                // some providers send invalid JSON RPC in the error case (no `id:u64`), but the
                // text should be a `JsonRpcError`
                #[derive(Deserialize)]
                struct Resp {
                    error: JsonRpcError,
                }

                if let Ok(resp) = serde_json::from_str::<Resp>(text) {
                    return self.should_retry_on_json_rpc_error(&resp.error);
                }
                false
            }

            // JSON-RPC error
            JsonRpcProviderClientError::JsonRpcError(err) => self.should_retry_on_json_rpc_error(err),

            // HTTP & transport errors: Only retry HTTP Too Many Requests and Timeouts
            JsonRpcProviderClientError::BackendError(HttpRequestError::Timeout)
            | JsonRpcProviderClientError::BackendError(HttpRequestError::HttpError(429)) => true,

            // Everything else is not retried and immediately considered an error
            _ => false,
        }
    }

    fn backoff_hint(&self, error: &JsonRpcProviderClientError) -> Option<Duration> {
        if let JsonRpcProviderClientError::JsonRpcError(JsonRpcError { data, .. }) = error {
            let data = data.as_ref()?;

            // if daily rate limit exceeded, infura returns the requested backoff in the error
            // response
            let backoff_seconds = &data["rate"]["backoff_seconds"];
            // infura rate limit error
            if let Some(seconds) = backoff_seconds.as_u64() {
                return Some(Duration::from_secs(seconds));
            }
            if let Some(seconds) = backoff_seconds.as_f64() {
                return Some(Duration::from_secs(seconds as u64 + 1));
            }
        }

        None
    }
}

pub mod native {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[cfg(feature = "reqwest")]
    use reqwest::header::{HeaderValue, CONTENT_TYPE};

    use crate::errors::HttpRequestError;
    use crate::HttpPostRequestor;

    /// Common configuration for all native `HttpPostRequestor`s
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct GeneralHttpPostRequestorConfig {
        /// Timeout for HTTP POST request
        /// Defaults to 5 seconds.
        pub http_request_timeout: Duration,
    }

    impl Default for GeneralHttpPostRequestorConfig {
        fn default() -> Self {
            Self {
                http_request_timeout: Duration::from_secs(5),
            }
        }
    }

    /// HTTP client that uses a non-Tokio runtime based HTTP client library, such as `surf`.
    /// `surf` works also for Browsers in WASM environments.
    #[derive(Clone, Debug)]
    pub struct SurfRequestor(surf::Client);

    impl SurfRequestor {
        pub fn new(cfg: GeneralHttpPostRequestorConfig) -> Self {
            let client = surf::Config::new()
                .set_timeout(Some(cfg.http_request_timeout))
                .try_into()
                .expect("failed to build Surf client from config"); // infallible for surf h1 client
            Self(client)
        }
    }

    impl Default for SurfRequestor {
        fn default() -> Self {
            Self::new(GeneralHttpPostRequestorConfig::default())
        }
    }

    #[async_trait]
    impl HttpPostRequestor for SurfRequestor {
        async fn http_post(&self, url: &str, json_data: &str) -> Result<String, HttpRequestError> {
            let mut body = surf::Body::from_string(json_data.to_owned());
            body.set_mime("application/json");

            //debug!("-> http post {url}: {json_data}");

            let mut response = self
                .0
                .post(url)
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(|e| HttpRequestError::HttpError(e.status().into()))?;

            let data = response
                .body_string()
                .await
                .map_err(|e| HttpRequestError::HttpError(e.status().into()))?;

            //debug!("<- http post response len={}", data.len());

            Ok(data)
        }
    }

    /// HTTP client that uses a Tokio-runtime based HTTP client library, such as `reqwest`.
    #[cfg(feature = "reqwest")]
    #[derive(Debug)]
    pub struct ReqwestRequestor(reqwest::Client);

    #[cfg(feature = "reqwest")]
    impl ReqwestRequestor {
        pub fn new(cfg: GeneralHttpPostRequestorConfig) -> Self {
            Self(
                reqwest::Client::builder()
                    .timeout(cfg.http_request_timeout)
                    .build()
                    .expect("failed to build Reqwest client"),
            )
        }
    }

    #[cfg(feature = "reqwest")]
    impl Default for ReqwestRequestor {
        fn default() -> Self {
            Self::new(GeneralHttpPostRequestorConfig::default())
        }
    }

    #[cfg(feature = "reqwest")]
    #[async_trait]
    impl HttpPostRequestor for ReqwestRequestor {
        async fn http_post(&self, url: &str, json_data: &str) -> Result<String, HttpRequestError> {
            //debug!("-> http post {url}: {json_data}");

            let resp = self
                .0
                .post(url)
                .header(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap())
                .body(Vec::from(json_data.as_bytes()))
                .send()
                .await
                .map_err(|e| {
                    if e.is_status() {
                        HttpRequestError::HttpError(e.status().unwrap().as_u16())
                    } else if e.is_timeout() {
                        HttpRequestError::Timeout
                    } else {
                        HttpRequestError::UnknownError(e.to_string())
                    }
                })?;

            let data = resp
                .text()
                .await
                .map_err(|e| HttpRequestError::UnknownError(format!("body: {}", e.to_string())))?;

            //debug!("<- http post response with {}", data.len());

            Ok(data)
        }
    }
}

/// Used for testing. Creates Ethers RPC client to the local Anvil instance.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_rpc_client_to_anvil<R: HttpPostRequestor + Debug>(
    backend: R,
    anvil: &ethers::utils::AnvilInstance,
    signer: &hopr_crypto::keypairs::ChainKeypair,
) -> std::sync::Arc<
    ethers::middleware::SignerMiddleware<
        ethers::providers::Provider<JsonRpcProviderClient<R>>,
        ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
    >,
> {
    use hopr_crypto::keypairs::Keypair;
    use ethers::signers::Signer;

    let wallet =
        ethers::signers::LocalWallet::from_bytes(signer.secret().as_ref()).expect("failed to construct wallet");
    let json_client = JsonRpcProviderClient::new(&anvil.endpoint(), backend);
    let provider = ethers::providers::Provider::new(json_client).interval(Duration::from_millis(10u64));

    std::sync::Arc::new(ethers::middleware::SignerMiddleware::new(
        provider,
        wallet.with_chain_id(anvil.chain_id()),
    ))
}

#[cfg(test)]
pub mod tests {
    use hopr_crypto::keypairs::{ChainKeypair, Keypair};
    use chain_types::{create_anvil, ContractAddresses, ContractInstances};
    use ethers_providers::JsonRpcClient;
    use futures::FutureExt;
    use std::time::Duration;
    use utils_types::primitives::Address;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient};
    use crate::errors::JsonRpcProviderClientError;
    use crate::MockHttpPostRequestor;

    #[async_std::test]
    async fn test_client_should_deploy_contracts() {
        let anvil = create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);

        let contract_addrs = ContractAddresses::from(
            &ContractInstances::deploy_for_testing(client.clone(), &chain_key_0)
                .await
                .expect("failed to deploy"),
        );

        assert_ne!(contract_addrs.token, Address::default());
        assert_ne!(contract_addrs.channels, Address::default());
        assert_ne!(contract_addrs.announcements, Address::default());
        assert_ne!(contract_addrs.network_registry, Address::default());
        assert_ne!(contract_addrs.safe_registry, Address::default());
        assert_ne!(contract_addrs.price_oracle, Address::default());
    }

    #[async_std::test]
    async fn test_client_should_get_block_number() {
        let block_time = Duration::from_secs(1);

        let anvil = create_anvil(Some(block_time));
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), SurfRequestor::default());

        let mut last_number = 0;

        for _ in 0..3 {
            async_std::task::sleep(block_time).await;

            let number: ethers::types::U64 = client
                .request("eth_blockNumber", ())
                .await
                .expect("should get block number");

            assert!(number.as_u64() > last_number, "next block number must be greater");
            last_number = number.as_u64();
        }
    }

    #[async_std::test]
    async fn test_client_should_fail_on_malformed_request() {
        let anvil = create_anvil(None);
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), SurfRequestor::default());

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber_bla", ())
            .await
            .expect_err("expected error");
        assert!(matches!(err, JsonRpcProviderClientError::JsonRpcError(..)));
    }

    #[async_std::test]
    async fn test_client_should_fail_on_malformed_response() {
        let mut mock_requestor = MockHttpPostRequestor::new();

        mock_requestor
            .expect_http_post()
            .once()
            .withf(|url, _| url == "localhost")
            .returning(|_, _| futures::future::ok("}{malformed".to_string()).boxed());

        let client = JsonRpcProviderClient::new("localhost".into(), mock_requestor);

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");
        assert!(matches!(err, JsonRpcProviderClientError::SerdeJson { .. }));
    }
}
