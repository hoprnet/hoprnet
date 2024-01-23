use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, JsonRpcError};
use log::{debug, trace, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use validator::Validate;

use crate::client::RetryAction::{NoRetry, RetryAfter};
use crate::errors::{HttpRequestError, JsonRpcProviderClientError};
use crate::helper::{Request, Response};
use crate::{HttpPostRequestor, RetryAction, RetryPolicy};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, MultiHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_RPC_CALLS: MultiCounter = MultiCounter::new(
        "hopr_rpc_call_count",
        "Number of Ethereum RPC calls over HTTP and their result",
        &["call", "result"]
    )
    .unwrap();
    static ref METRIC_RPC_CALLS_TIMING: MultiHistogram = MultiHistogram::new(
        "hopr_rpc_call_time_sec",
        "Timing of RPC calls over HTTP in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 7.0, 10.0],
        &["call"]
    )
    .unwrap();
    static ref METRIC_RETRIES_PER_RPC_CALL: MultiHistogram = MultiHistogram::new(
        "hopr_retries_per_rpc_call",
        "Number of retries per RPC call",
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        &["call"]
    )
    .unwrap();
}

/// Defines a retry policy suitable for `JsonRpcProviderClient`.
/// This retry policy distinguishes between 4 types of RPC request failures:
/// - JSON RPC error (based on error code)
/// - HTTP error (based on HTTP status)
/// - Transport error (e.g. connection timeout)
/// - Serde error (some of these are treated as JSON RPC error above, if an error code can be obtained).
/// The policy will make up to `max_retries` once a JSON RPC request fails.
/// Each retry `k > 0` will be separated by a delay of `initial_backoff * (1 + backoff_coefficient)^(k - 1)`,
/// namely all the JSON RPC error codes specified in `retryable_json_rpc_errors` and all the HTTP errors
/// specified in `retryable_http_errors`.
/// Transport and connection errors (such as connection timeouts) are retried without backoff
/// at a constant delay of `initial_backoff` if `backoff_on_transport_errors` is not set.
/// No more additional retries are allowed on new requests, if the maximum number of concurrent
/// requests being retried has reached `max_retry_queue_size`.
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct SimpleJsonRpcRetryPolicy {
    /// Maximum number of retries.
    /// If `None` is given, will keep retrying indefinitely.
    /// Default is 10.
    #[validate(range(min = 1))]
    pub max_retries: Option<u32>,
    /// Initial wait before retries.
    /// NOTE: Transport and connection errors (such as connection timeouts) are retried at
    /// a constant rate (no backoff) with this delay if `backoff_on_transport_errors` is not set.
    /// Default is 1 second.
    pub initial_backoff: Duration,
    /// Backoff coefficient by which will be each retry multiplied.
    /// Must be non-negative. If set to `0`, no backoff will be applied and the
    /// requests will be retried at a constant rate.
    /// Default is 1.001
    #[validate(range(min = 0))]
    pub backoff_coefficient: f64,
    /// Maximum backoff value.
    /// Once reached, the requests will be retried at a constant rate with this timeout.
    /// Default is 120 seconds.
    pub max_backoff: Duration,
    /// Indicates whether to also apply backoff to transport and connection errors (such as connection timeouts).
    /// Default is false.
    pub backoff_on_transport_errors: bool,
    /// List of JSON RPC errors that should be retried with backoff
    /// Default is [429, -32005, -32016]
    pub retryable_json_rpc_errors: Vec<i64>,
    /// List of HTTP errors that should be retried with backoff.
    /// Default is [429]
    pub retryable_http_errors: Vec<http_types::StatusCode>,
    /// Maximum number of different requests that are being retried at the same time.
    /// If any additional request fails after this number is attained, it won't be retried.
    /// Defaults to 3
    #[validate(range(min = 1))]
    pub max_retry_queue_size: u32,
}

impl Default for SimpleJsonRpcRetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: Some(10),
            initial_backoff: Duration::from_secs(1),
            backoff_coefficient: 1.001,
            max_backoff: Duration::from_secs(120),
            backoff_on_transport_errors: false,
            retryable_json_rpc_errors: vec![-32005, -32016, 429],
            retryable_http_errors: vec![http_types::StatusCode::TooManyRequests],
            max_retry_queue_size: 3,
        }
    }
}

impl SimpleJsonRpcRetryPolicy {
    fn is_retryable_json_rpc_error(&self, err: &JsonRpcError) -> bool {
        self.retryable_json_rpc_errors.contains(&err.code) || err.message.contains("rate limit")
    }

    fn is_retryable_http_error(&self, status: &http_types::StatusCode) -> bool {
        self.retryable_http_errors.contains(status)
    }
}

impl RetryPolicy<JsonRpcProviderClientError> for SimpleJsonRpcRetryPolicy {
    fn is_retryable_error(
        &self,
        err: &JsonRpcProviderClientError,
        num_retries: u32,
        retry_queue_size: u32,
    ) -> RetryAction {
        if self.max_retries.is_some_and(|max| num_retries > max) {
            warn!("max number of retries {} has been reached", self.max_retries.unwrap());
            return NoRetry;
        }

        if retry_queue_size > self.max_retry_queue_size {
            warn!(
                "maximum size of retry queue {} has been reached",
                self.max_retry_queue_size
            );
            return NoRetry;
        }

        // next_backoff = initial_backoff * (1 + backoff_coefficient)^(num_retries - 1)
        let backoff = self
            .initial_backoff
            .mul_f64(f64::powi(1.0 + self.backoff_coefficient, (num_retries - 1) as i32))
            .min(self.max_backoff);

        match err {
            // Retryable JSON RPC errors are retries with backoff
            JsonRpcProviderClientError::JsonRpcError(e) if self.is_retryable_json_rpc_error(e) => {
                debug!("encountered retryable JSON RPC error code: {e}");
                RetryAfter(backoff)
            }

            // Retryable HTTP errors are retries with backoff
            JsonRpcProviderClientError::BackendError(HttpRequestError::HttpError(e))
                if self.is_retryable_http_error(e) =>
            {
                debug!("encountered retryable HTTP error code: {e}");
                RetryAfter(backoff)
            }

            // Transport error and timeouts are retried at a constant rate if specified
            JsonRpcProviderClientError::BackendError(e @ HttpRequestError::Timeout)
            | JsonRpcProviderClientError::BackendError(e @ HttpRequestError::TransportError(_))
            | JsonRpcProviderClientError::BackendError(e @ HttpRequestError::UnknownError(_)) => {
                debug!("encountered retryable transport error: {e}");
                RetryAfter(if self.backoff_on_transport_errors {
                    backoff
                } else {
                    self.initial_backoff
                })
            }

            // Some providers send invalid JSON RPC in the error case (no `id:u64`), but the text is a `JsonRpcError`
            JsonRpcProviderClientError::SerdeJson { text, .. } => {
                #[derive(Deserialize)]
                struct Resp {
                    error: JsonRpcError,
                }

                match serde_json::from_str::<Resp>(text) {
                    Ok(Resp { error }) if self.is_retryable_json_rpc_error(&error) => {
                        debug!("encountered retryable JSON RPC error: {error}");
                        RetryAfter(backoff)
                    }
                    _ => {
                        debug!("unparseable JSON RPC error: {text}");
                        NoRetry
                    }
                }
            }

            // Anything else is not retried
            _ => NoRetry,
        }
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
    retry_policy: R,
}

impl<Req: HttpPostRequestor, R: RetryPolicy<JsonRpcProviderClientError>> JsonRpcProviderClient<Req, R> {
    /// Creates the client given the `HttpPostRequestor`
    pub fn new(base_url: &str, requestor: Req, retry_policy: R) -> Self {
        Self {
            id: AtomicU64::new(1),
            requests_enqueued: AtomicU32::new(0),
            url: base_url.to_owned(),
            requestor,
            retry_policy,
        }
    }

    async fn send_request_internal<T, A>(&self, method: &str, params: T) -> Result<A, JsonRpcProviderClientError>
    where
        T: Serialize + Send + Sync,
        A: DeserializeOwned,
    {
        // Create the Request object
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);

        // Perform the actual request
        let start = std::time::Instant::now();
        let body = self.requestor.http_post(self.url.as_ref(), payload).await?;
        let req_duration = start.elapsed();

        debug!("rpc call {method} took {}ms", req_duration.as_millis());

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_RPC_CALLS_TIMING.observe(&[method], req_duration.as_secs_f64());

        // First deserialize the Response object
        let raw = match serde_json::from_slice(&body) {
            Ok(Response::Success { result, .. }) => result.to_owned(),
            Ok(Response::Error { error, .. }) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(error.into());
            }
            Ok(_) => {
                let err = JsonRpcProviderClientError::SerdeJson {
                    err: serde::de::Error::custom("unexpected notification over HTTP transport"),
                    text: String::from_utf8_lossy(&body).to_string(),
                };
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(err);
            }
            Err(err) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_RPC_CALLS.increment(&[method, "failure"]);

                return Err(JsonRpcProviderClientError::SerdeJson {
                    err,
                    text: String::from_utf8_lossy(&body).to_string(),
                });
            }
        };

        // Next, deserialize the data out of the Response object
        let res = serde_json::from_str(raw.get()).map_err(|err| JsonRpcProviderClientError::SerdeJson {
            err,
            text: raw.to_string(),
        })?;

        #[cfg(all(feature = "prometheus", not(test)))]
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

impl<Req: HttpPostRequestor + Clone, R: RetryPolicy<JsonRpcProviderClientError> + Clone> Clone
    for JsonRpcProviderClient<Req, R>
{
    fn clone(&self) -> Self {
        Self {
            id: AtomicU64::new(1),
            url: self.url.clone(),
            requests_enqueued: AtomicU32::new(0),
            requestor: self.requestor.clone(),
            retry_policy: self.retry_policy.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Req, R> JsonRpcClient for JsonRpcProviderClient<Req, R>
where
    Req: HttpPostRequestor,
    R: RetryPolicy<JsonRpcProviderClientError> + Send + Sync,
{
    type Error = JsonRpcProviderClientError;

    async fn request<T, A>(&self, method: &str, params: T) -> Result<A, Self::Error>
    where
        T: Serialize + Send + Sync,
        A: DeserializeOwned + Send,
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
            let params = serde_json::to_value(params)
                .map_err(|err| JsonRpcProviderClientError::SerdeJson { err, text: "".into() })?;
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

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_RETRIES_PER_RPC_CALL.observe(&[method], num_retries as f64);

                        return Ok(ret);
                    }
                    Err(req_err) => {
                        err = req_err;
                        num_retries += 1;
                    }
                }
            }

            match self
                .retry_policy
                .is_retryable_error(&err, num_retries, self.requests_enqueued.load(Ordering::SeqCst))
            {
                NoRetry => {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RETRIES_PER_RPC_CALL.observe(&[method], num_retries as f64);

                    return Err(err);
                }
                RetryAfter(backoff) => async_std::task::sleep(backoff).await,
            }
        }
    }
}

pub mod native {
    use async_std::prelude::FutureExt;
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

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
        cfg: HttpPostRequestorConfig,
    }

    impl SurfRequestor {
        pub fn new(cfg: HttpPostRequestorConfig) -> Self {
            // Here we do not set the timeout into the client's configuration
            // but rather the timeout is set on the entire Future in `http_post`
            Self {
                client: surf::client(),
                cfg,
            }
        }
    }

    #[async_trait]
    impl HttpPostRequestor for SurfRequestor {
        async fn http_post<T: Serialize + Send + Sync>(
            &self,
            url: &str,
            data: T,
        ) -> Result<Box<[u8]>, HttpRequestError> {
            let request = self
                .client
                .post(url)
                .body_json(&data)
                .map_err(|e| HttpRequestError::UnknownError(e.to_string()))?;

            async move {
                match request.await {
                    Ok(mut response) if response.status().is_success() => match response.body_bytes().await {
                        Ok(data) => Ok(data.into_boxed_slice()),
                        Err(e) => Err(HttpRequestError::TransportError(e.to_string())),
                    },
                    Ok(response) => Err(HttpRequestError::HttpError(response.status())),
                    Err(e) => Err(HttpRequestError::TransportError(e.to_string())),
                }
            }
            .timeout(self.cfg.http_request_timeout)
            .await
            .map_err(|_| HttpRequestError::Timeout)?
        }
    }
}

type AnvilRpcClient<R> = std::sync::Arc<
    ethers::middleware::SignerMiddleware<
        ethers::providers::Provider<JsonRpcProviderClient<R, SimpleJsonRpcRetryPolicy>>,
        ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
    >,
>;

/// Used for testing. Creates Ethers RPC client to the local Anvil instance.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_rpc_client_to_anvil<R: HttpPostRequestor + Debug>(
    backend: R,
    anvil: &ethers::utils::AnvilInstance,
    signer: &hopr_crypto_types::keypairs::ChainKeypair,
) -> AnvilRpcClient<R> {
    use ethers::signers::Signer;
    use hopr_crypto_types::keypairs::Keypair;

    let wallet =
        ethers::signers::LocalWallet::from_bytes(signer.secret().as_ref()).expect("failed to construct wallet");
    let json_client = JsonRpcProviderClient::new(&anvil.endpoint(), backend, SimpleJsonRpcRetryPolicy::default());
    let provider = ethers::providers::Provider::new(json_client).interval(Duration::from_millis(10_u64));

    std::sync::Arc::new(ethers::middleware::SignerMiddleware::new(
        provider,
        wallet.with_chain_id(anvil.chain_id()),
    ))
}

#[cfg(test)]
pub mod tests {
    use chain_types::{create_anvil, ContractAddresses, ContractInstances};
    use ethers_providers::JsonRpcClient;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::primitives::Address;
    use serde_json::json;
    use std::time::Duration;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};
    use crate::errors::JsonRpcProviderClientError;
    use crate::ZeroRetryPolicy;

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
        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

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
        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber_bla", ())
            .await
            .expect_err("expected error");

        assert!(matches!(err, JsonRpcProviderClientError::JsonRpcError(..)));
    }

    #[async_std::test]
    async fn test_client_should_fail_on_malformed_response() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("}malformed{")
            .expect(1)
            .create();

        let client = JsonRpcProviderClient::new(
            &server.url(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::SerdeJson { .. }));
    }

    #[async_std::test]
    async fn test_client_should_retry_on_http_error() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(http_types::StatusCode::TooManyRequests as usize)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("{}")
            .expect(3)
            .create();

        let client = JsonRpcProviderClient::new(
            &server.url(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy {
                max_retries: Some(2),
                retryable_http_errors: vec![http_types::StatusCode::TooManyRequests],
                initial_backoff: Duration::from_millis(100),
                ..SimpleJsonRpcRetryPolicy::default()
            },
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::BackendError(_)));
    }

    #[async_std::test]
    async fn test_client_should_not_retry_with_zero_retry_policy() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(404)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("{}")
            .expect(1)
            .create();

        let client = JsonRpcProviderClient::new(&server.url(), SurfRequestor::default(), ZeroRetryPolicy::default());

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::BackendError(_)));
    }

    #[async_std::test]
    async fn test_client_should_retry_on_json_rpc_error() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body(
                r#"{
              "jsonrpc": "2.0",
              "id": 1,
              "error": {
                "message": "some message",
                "code": -32603
              }
            }"#,
            )
            .expect(3)
            .create();

        let client = JsonRpcProviderClient::new(
            &server.url(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy {
                max_retries: Some(2),
                retryable_json_rpc_errors: vec![-32603],
                initial_backoff: Duration::from_millis(100),
                ..SimpleJsonRpcRetryPolicy::default()
            },
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::JsonRpcError(_)));
    }

    #[async_std::test]
    async fn test_client_should_not_retry_on_nonretryable_json_rpc_error() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body(
                r#"{
              "jsonrpc": "2.0",
              "id": 1,
              "error": {
                "message": "some message",
                "code": -32000
              }
            }"#,
            )
            .expect(1)
            .create();

        let client = JsonRpcProviderClient::new(
            &server.url(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy {
                max_retries: Some(2),
                retryable_json_rpc_errors: vec![],
                initial_backoff: Duration::from_millis(100),
                ..SimpleJsonRpcRetryPolicy::default()
            },
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::JsonRpcError(_)));
    }

    #[async_std::test]
    async fn test_client_should_retry_on_malformed_json_rpc_error() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body(
                r#"{
              "jsonrpc": "2.0",
              "error": {
                "message": "some message",
                "code": -32600
              }
            }"#,
            )
            .expect(3)
            .create();

        let client = JsonRpcProviderClient::new(
            &server.url(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy {
                max_retries: Some(2),
                retryable_json_rpc_errors: vec![-32600],
                initial_backoff: Duration::from_millis(100),
                ..SimpleJsonRpcRetryPolicy::default()
            },
        );

        let err = client
            .request::<_, ethers::types::U64>("eth_blockNumber", ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, JsonRpcProviderClientError::SerdeJson { .. }));
    }
}
