use async_trait::async_trait;
use ethers_providers::JsonRpcClient;
use log::trace;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use crate::errors::JsonRpcProviderClientError;
use crate::helper::{Request, Response};
use crate::{HttpPostRequestor, RetryAction, RetryPolicy};

#[cfg(feature = "prometheus")]
use metrics::metrics::{MultiCounter, MultiHistogram};
use crate::client::RetryAction::{NoRetry, RetryAfter};

#[cfg(feature = "prometheus")]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_RPC_CALLS: MultiCounter = MultiCounter::new(
        "hopr_rpc_call_count",
        "Number of Ethereum RPC calls over HTTP and their result",
        &["call", "result"]
    )
    .unwrap();
    static ref METRIC_COUNT_RPC_CALLS_TIMING: MultiHistogram = MultiHistogram::new(
        "hopr_rpc_call_time_sec",
        "Timing of RPC calls over HTTP in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 7.0, 10.0],
        &["call"]
    )
    .unwrap();
}

pub struct JsonRpcSimpleRetryPolicy;

impl RetryPolicy<JsonRpcProviderClientError> for JsonRpcSimpleRetryPolicy {
    fn is_retryable_error(&self, err: &JsonRpcProviderClientError, num_retries: u32, retry_queue_size: u32) -> RetryAction {
        todo!()
    }
}

/// Modified implementation of `ethers::providers::Http` so that it can
/// operate with any `HttpPostRequestor`.
/// Also contains possible retry actions to be taken on various failures, therefore it
/// implements also `ethers::providers::RetryClient` functionality.
pub struct JsonRpcProviderClient<Req: HttpPostRequestor, R: RetryPolicy<JsonRpcProviderClientError>> {
    id: AtomicU64,
    requests_enqueued: AtomicU32,
    url: String,
    requestor: Req,
    retry_policy: R
}

impl<Req: HttpPostRequestor, R: RetryPolicy<JsonRpcProviderClientError>> JsonRpcProviderClient<Req, R> {
    /// Creates the client given the `HttpPostRequestor`
    pub fn new(base_url: &str, requestor: Req, retry_policy: R) -> Self {
        Self {
            id: AtomicU64::new(1),
            requests_enqueued: AtomicU32::new(0),
            url: base_url.to_owned(),
            requestor,
            retry_policy
        }
    }

    async fn send_request_internal<T, A>(&self, method: &str, params: T) -> Result<A, JsonRpcProviderClientError>
        where
            T: Serialize + Send + Sync,
            A: DeserializeOwned
    {
        // Create the Request object
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);

        // Perform the actual request
        let start = std::time::Instant::now();
        let body = self.requestor.http_post(self.url.as_ref(), payload).await?;
        let req_duration = start.elapsed();

        trace!("rpc call {method} took {}ms", req_duration.as_millis());

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS_TIMING.observe(&[method], req_duration.as_secs_f64());

        // First deserialize the Response object
        let raw = match serde_json::from_slice(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => {
                #[cfg(feature = "prometheus")]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(error.into());
            }
            Ok(_) => {
                let err = JsonRpcProviderClientError::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: String::from_utf8_lossy(&body).to_string(),
                };
                #[cfg(feature = "prometheus")]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(err);
            }
            Err(err) => {
                #[cfg(feature = "prometheus")]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(JsonRpcProviderClientError::SerdeJson { err, text: String::from_utf8_lossy(&body).to_string() });
            }
        };

        // Next, deserialize the data out of the Response object
        let res = serde_json::from_str(raw.get()).map_err(|err| JsonRpcProviderClientError::SerdeJson {
            err,
            text: raw.to_string(),
        })?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment(&[method, "success"]);

        Ok(res)
    }
}

impl<Req: HttpPostRequestor, R: RetryPolicy<JsonRpcProviderClientError>> Debug for JsonRpcProviderClient<Req, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonRpcProviderClient")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("requests_enqueued", &self.requests_enqueued)
            .finish_non_exhaustive()
    }
}

impl<Req: HttpPostRequestor + Clone, R: RetryPolicy<JsonRpcProviderClientError> + Clone> Clone for JsonRpcProviderClient<Req, R> {
    fn clone(&self) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: self.url.clone(),
            requests_enqueued: AtomicU32::new(0),
            requestor: self.requestor.clone(),
            retry_policy: self.retry_policy.clone()
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Req, R> JsonRpcClient for JsonRpcProviderClient<Req, R>
where Req: HttpPostRequestor, R: RetryPolicy<JsonRpcProviderClientError> + Send + Sync {
    type Error = JsonRpcProviderClientError;

    async fn request<T, A>(&self, method: &str, params: T) -> Result<A, Self::Error>
    where
        T: Serialize + Send + Sync,
        A: DeserializeOwned + Send
    {
        // Helper type that caches the `params` value across several retries
        // This is necessary because the wrapper provider is supposed to skip he `params` if it's of
        // size 0, see `crate::transports::common::Request`
        enum RetryParams<Params> {
            Value(Params),
            Zst(()),
        }

        let params = if std::mem::size_of::<A>() == 0 {
            RetryParams::Zst(())
        } else {
            let params = serde_json::to_value(params).map_err(|err| JsonRpcProviderClientError::SerdeJson{
                err,
                text: "".into()
            })?;
            RetryParams::Value(params)
        };

        let mut num_retries = 0;
        loop {
            let err;

            // hack to not hold `A` across an await in the sleep future and prevent requiring
            // A: Send + Sync
            {
                let resp = match params {
                    RetryParams::Value(ref params) => self.send_request_internal(method, params).await,
                    RetryParams::Zst(unit) => self.send_request_internal(method, unit).await,
                };
                match resp {
                    Ok(ret) => {
                        self.requests_enqueued.fetch_sub(1, Ordering::SeqCst);
                        return Ok(ret)
                    }
                    Err(req_err) => err = req_err,
                }
            }

            match self.retry_policy.is_retryable_error(&err, num_retries, self.requests_enqueued.load(Ordering::SeqCst)) {
                NoRetry => return Err(err),
                RetryAfter(backoff) => {
                    num_retries += 1;
                    async_std::task::sleep(backoff).await;
                }
            }
        }
    }
}

pub mod native {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use async_std::prelude::FutureExt;
    use futures::TryFutureExt;

    use crate::errors::HttpRequestError;
    use crate::HttpPostRequestor;

    /// Common configuration for all native `HttpPostRequestor`s
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct HttpPostRequestorConfig {
        /// Timeout for HTTP POST request
        /// Defaults to 5 seconds.
        pub http_request_timeout: Duration,
    }

    impl Default for HttpPostRequestorConfig {
        fn default() -> Self {
            Self {
                http_request_timeout: Duration::from_secs(5),
            }
        }
    }

    /// HTTP client that uses a non-Tokio runtime based HTTP client library, such as `surf`.
    /// `surf` works also for Browsers in WASM environments.
    #[derive(Clone, Debug, Default)]
    pub struct SurfRequestor {
        client: surf::Client,
        cfg: HttpPostRequestorConfig
    }

    impl SurfRequestor {
        pub fn new(cfg: HttpPostRequestorConfig) -> Self {
            // Here we do not set the timeout into the client's configuration
            // but rather the timeout is set on the entire Future in `http_post`
            Self {
                client: surf::client(),
                cfg
            }
        }
    }

    #[async_trait]
    impl HttpPostRequestor for SurfRequestor {
        async fn http_post<T: Serialize + Send + Sync>(&self, url: &str, data: T) -> Result<Box<[u8]>, HttpRequestError> {
            let result = self
                .client
                .post(url)
                .body_json(&data)
                .map_err(|e| HttpRequestError::UnknownError(e.to_string()))?
                .and_then(|mut resp| async move {
                    resp.body_bytes().await
                })
                .timeout(self.cfg.http_request_timeout) // set global request timeout on the future
                .await;

            match result {
                Ok(Ok(data)) => Ok(data.into_boxed_slice()),
                Ok(Err(e)) => Err(HttpRequestError::HttpError(e.status().into())),
                Err(_) => Err(HttpRequestError::Timeout)
            }
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
        ethers::providers::Provider<JsonRpcProviderClient<R, JsonRpcSimpleRetryPolicy>>,
        ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
    >,
> {
    use ethers::signers::Signer;
    use hopr_crypto::keypairs::Keypair;

    let wallet =
        ethers::signers::LocalWallet::from_bytes(signer.secret().as_ref()).expect("failed to construct wallet");
    let json_client = JsonRpcProviderClient::new(&anvil.endpoint(), backend, JsonRpcSimpleRetryPolicy);
    let provider = ethers::providers::Provider::new(json_client).interval(Duration::from_millis(10u64));

    std::sync::Arc::new(ethers::middleware::SignerMiddleware::new(
        provider,
        wallet.with_chain_id(anvil.chain_id()),
    ))
}

#[cfg(test)]
pub mod tests {
    use chain_types::{create_anvil, ContractAddresses, ContractInstances};
    use ethers_providers::JsonRpcClient;
    use futures::FutureExt;
    use hopr_crypto::keypairs::{ChainKeypair, Keypair};
    use std::time::Duration;
    use utils_types::primitives::Address;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, JsonRpcSimpleRetryPolicy};
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
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), SurfRequestor::default(), JsonRpcSimpleRetryPolicy);

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
        let client = JsonRpcProviderClient::new(&anvil.endpoint(), SurfRequestor::default(), JsonRpcSimpleRetryPolicy);

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

        let client = JsonRpcProviderClient::new("localhost".into(), mock_requestor, JsonRpcSimpleRetryPolicy);

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");
        assert!(matches!(err, JsonRpcProviderClientError::SerdeJson { .. }));
    }
}
