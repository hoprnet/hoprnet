//! Extended layers of RPC clients:
//! - Replace the legacy retry backoff layer with the default [`RetryBackoffService`]
//! - Add Metrics Layer
//!
//! Extended `JsonRpcClient` abstraction.
//!
//! This module contains custom implementation of `ethers::providers::JsonRpcClient`
//! which allows usage of non-`reqwest` based HTTP clients.
//!
//! The major type implemented in this module is the [JsonRpcProviderClient]
//! which implements the [ethers::providers::JsonRpcClient] trait. That makes it possible to use it with `ethers`.
//!
//! The [JsonRpcProviderClient] is abstract over the [HttpRequestor] trait, which makes it possible
//! to make the underlying HTTP client implementation easily replaceable. This is needed to make it possible
//! for `ethers` to work with different async runtimes, since the HTTP client is typically not agnostic to
//! async runtimes (the default HTTP client in `ethers` is using `reqwest`, which is `tokio` specific).
//! Secondly, this abstraction also allows implementing WASM-compatible HTTP client if needed at some point.

use alloy::{
    rpc::json_rpc::{ErrorPayload, RequestPacket, ResponsePacket, ResponsePayload},
    transports::{layers::RetryPolicy, HttpError, TransportError, TransportErrorKind, TransportFut},
};
// use async_trait::async_trait;
// use ethers::providers::{JsonRpcClient, JsonRpcError};
use futures::StreamExt;
// use http_types::Method;
// use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};
use tower::{Layer, Service};
use tracing::{error, trace};
use validator::Validate;

// use hopr_async_runtime::prelude::sleep;

// use crate::client::RetryAction::{NoRetry, RetryAfter};
// use crate::errors::{HttpRequestError, JsonRpcProviderClientError};
// use crate::helper::{Request, Response};
// use crate::{RetryAction, RetryPolicy};
// use crate::{HttpRequestor};

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

/// Defines a default retry policy suitable for `RpcClient`.
/// This is a reimplementation of the legacy "retry policy suitable for `JsonRpcProviderClient`"
///
/// This retry policy distinguishes between 4 types of RPC request failures:
/// - JSON RPC error (based on error code)
/// - HTTP error (based on HTTP status)
/// - Transport error (e.g. connection timeout)
/// - Serde error (some of these are treated as JSON RPC error above, if an error code can be obtained).
///
/// The standard `RetryBackoffLayer` defines the following properties:
/// - `max_rate_limit_retries`: (u32) The maximum number of retries for rate limit errors.
/// Different from the legacy implementation, there is always an upper limit.
/// - `initial_backoff`: (u64) The initial backoff in milliseconds
/// - `compute_units_per_second`: (u64) The number of compute units per second for this service
///
/// The policy will make up to `max_retries` once a JSON RPC request fails.
/// The minimum number of retries `min_retries` can be also specified and applies to any type of error regardless.
/// Each retry `k > 0` will be separated by a delay of `initial_backoff * (1 + backoff_coefficient)^(k - 1)`,
/// namely all the JSON RPC error codes specified in `retryable_json_rpc_errors` and all the HTTP errors
/// specified in `retryable_http_errors`.
///
/// The total wait time will be `(initial_backoff/backoff_coefficient) * ((1 + backoff_coefficient)^max_retries - 1)`.
/// or `max_backoff`, whatever is lower.
///
/// Transport and connection errors (such as connection timeouts) are retried without backoff
/// at a constant delay of `initial_backoff` if `backoff_on_transport_errors` is not set.
///
/// No more additional retries are allowed on new requests, if the maximum number of concurrent
/// requests being retried has reached `max_retry_queue_size`.
// #[derive(Clone, Debug, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
// #[derive(Debug, Clone)]
#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
pub struct DefaultRetryPolicy {
    /// Minimum number of retries of any error, regardless the error code.
    ///
    /// Default is 0.
    #[validate(range(min = 0))]
    #[default(Some(0))]
    pub min_retries: Option<u32>,

    // TODO: This property is on the outter layer
    // /// Maximum number of retries.
    // ///
    // /// If `None` is given, will keep retrying indefinitely.
    // ///
    // /// Default is 12.
    // #[validate(range(min = 1))]
    // #[default(Some(12))]
    // pub max_retries: Option<u32>,
    /// Initial wait before retries.
    ///
    /// NOTE: Transport and connection errors (such as connection timeouts) are retried at
    /// a constant rate (no backoff) with this delay if `backoff_on_transport_errors` is not set.
    ///
    /// Default is 1 second.
    #[default(Duration::from_secs(1))]
    pub initial_backoff: Duration,

    /// Backoff coefficient by which will be each retry multiplied.
    ///
    /// Must be non-negative. If set to `0`, no backoff will be applied and the
    /// requests will be retried at a constant rate.
    ///
    /// Default is 0.3
    #[validate(range(min = 0.0))]
    #[default(0.3)]
    pub backoff_coefficient: f64,
    /// Maximum backoff value.
    ///
    /// Once reached, the requests will be retried at a constant rate with this timeout.
    ///
    /// Default is 30 seconds.
    #[default(Duration::from_secs(30))]
    pub max_backoff: Duration,
    /// Indicates whether to also apply backoff to transport and connection errors (such as connection timeouts).
    ///
    /// Default is false.
    pub backoff_on_transport_errors: bool,
    /// List of JSON RPC errors that should be retried with backoff
    ///
    /// Default is \[429, -32005, -32016\]
    #[default(_code = "vec![-32005, -32016, 429]")]
    pub retryable_json_rpc_errors: Vec<i64>,

    /// List of HTTP errors that should be retried with backoff.
    ///
    /// Default is \[429, 504, 503\]
    #[default(
        _code = "vec![http_types::StatusCode::TooManyRequests,http_types::StatusCode::GatewayTimeout,http_types::StatusCode::ServiceUnavailable]"
    )]
    pub retryable_http_errors: Vec<http_types::StatusCode>,
    /// Maximum number of different requests that are being retried at the same time.
    ///
    /// If any additional request fails after this number is attained, it won't be retried.
    ///
    /// Default is 100
    #[validate(range(min = 5))]
    #[default = 100]
    pub max_retry_queue_size: u32,
}

impl DefaultRetryPolicy {
    fn is_retryable_json_rpc_errors(&self, rpc_err: &ErrorPayload) -> bool {
        self.retryable_json_rpc_errors.contains(&rpc_err.code)
    }

    fn is_retryable_http_errors(&self, http_err: &HttpError) -> bool {
        let status_code = match http_types::StatusCode::try_from(http_err.status) {
            Ok(status_code) => status_code,
            Err(_) => return false,
        };
        self.retryable_http_errors.contains(&status_code)
    }
}

impl RetryPolicy for DefaultRetryPolicy {
    // TODO: original implementation requires input param of `num_retries`
    fn should_retry(&self, err: &TransportError) -> bool {
        // // Retry if a global minimum of number of retries was given and wasn't yet attained
        // if self.min_retries.is_some_and(|min| num_retries <= min) {
        //     debug!(num_retries, min_retries = ?self.min_retries,  "retrying because minimum number of retries not yet reached");
        //     return true;
        // }
        match err {
            // There was a transport-level error. This is either a non-retryable error,
            // or a server error that should be retried.
            TransportError::Transport(err) => {
                match err {
                    // Missing batch response errors can be retried.
                    TransportErrorKind::MissingBatchResponse(_) => true,
                    TransportErrorKind::HttpError(http_err) => {
                        http_err.is_rate_limit_err() || self.is_retryable_http_errors(http_err)
                    }
                    TransportErrorKind::Custom(err) => {
                        let msg = err.to_string();
                        msg.contains("429 Too Many Requests")
                    }
                    _ => false,
                }
            }
            // The transport could not serialize the error itself. The request was malformed from
            // the start.
            TransportError::SerError(_) => false,
            TransportError::DeserError { text, .. } => {
                if let Ok(resp) = serde_json::from_str::<ErrorPayload>(text) {
                    return self.is_retryable_json_rpc_errors(&resp);
                }

                // some providers send invalid JSON RPC in the error case (no `id:u64`), but the
                // text should be a `JsonRpcError`
                #[derive(Deserialize)]
                struct Resp {
                    error: ErrorPayload,
                }

                if let Ok(resp) = serde_json::from_str::<Resp>(text) {
                    return self.is_retryable_json_rpc_errors(&resp.error);
                }

                false
            }
            TransportError::ErrorResp(err) => self.is_retryable_json_rpc_errors(err),
            TransportError::NullResp => true,
            _ => false,
        }
    }

    // TODO: original implementation requires input param of `num_retries`
    // next_backoff = initial_backoff * (1 + backoff_coefficient)^(num_retries - 1)
    fn backoff_hint(&self, _error: &alloy::transports::TransportError) -> Option<std::time::Duration> {
        // let backoff = self
        //     .initial_backoff
        //     .mul_f64(f64::powi(1.0 + self.backoff_coefficient, (num_retries - 1) as i32))
        //     .min(self.max_backoff);
        // Some(backoff)
        None
    }
}

#[derive(Debug, Clone)]
pub struct ZeroRetryPolicy;
impl RetryPolicy for ZeroRetryPolicy {
    fn should_retry(&self, _err: &alloy::transports::TransportError) -> bool {
        false
    }

    fn backoff_hint(&self, _error: &alloy::transports::TransportError) -> Option<std::time::Duration> {
        None
    }
}

pub struct MetricsLayer;

#[derive(Debug, Clone)]
pub struct MetricsService<S> {
    inner: S,
}

// Implement tower::Layer for MetricsLayer.
impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner }
    }
}

/// Implement the [`tower::Service`] trait for the [`MetricsService`].
impl<S> Service<RequestPacket> for MetricsService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError> + Send + 'static + Clone,
    S::Error: Send + 'static + Debug,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        // metrics before calling
        let start = std::time::Instant::now();

        let method_names = match request.clone() {
            RequestPacket::Single(single_req) => vec![single_req.method().to_owned()],
            RequestPacket::Batch(vec_req) => vec_req.iter().map(|s_req| s_req.method().to_owned()).collect(),
        };

        let future = self.inner.call(request);

        // metrics after calling
        Box::pin(async move {
            let res = future.await;

            let req_duration = start.elapsed();
            method_names.iter().for_each(|method| {
                trace!(method, duration_in_ms = req_duration.as_millis(), "rpc request took");
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RPC_CALLS_TIMING.observe(&[method], req_duration.as_secs_f64());
            });

            // First deserialize the Response object
            match &res {
                Ok(result) => match result {
                    ResponsePacket::Single(a) => match a.payload {
                        ResponsePayload::Success(_) => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_COUNT_RPC_CALLS.increment(&[&method_names[0], "success"]);
                        }
                        ResponsePayload::Failure(_) => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_COUNT_RPC_CALLS.increment(&[&method_names[0], "failure"]);
                        }
                    },
                    ResponsePacket::Batch(b) => {
                        b.iter().enumerate().for_each(|(i, _)| match b[i].payload {
                            ResponsePayload::Success(_) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_COUNT_RPC_CALLS.increment(&[&method_names[i], "success"]);
                            }
                            ResponsePayload::Failure(_) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_COUNT_RPC_CALLS.increment(&[&method_names[i], "failure"]);
                            }
                        });
                    }
                },
                Err(err) => {
                    error!(error = ?err, "Error occurred while processing request");
                    method_names.iter().for_each(|m| {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_RPC_CALLS.increment(&[&m, "failure"]);
                    });
                }
            };

            res
        })
    }
}

/// Snapshot of a response cached by the [`SnapshotRequestorLayer`].
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestorResponseSnapshot {
    id: usize,
    request: String,
    response: String,
}

/// Replays an RPC response to a request if it is found in the snapshot YAML file.
/// If no such request has been seen before,
/// it captures the new request/response pair obtained from the inner [`HttpRequestor`]
/// and stores it into the snapshot file.
///
/// This is useful for snapshot testing only and should **NOT** be used in production.
#[derive(Debug, Clone)]
pub struct SnapshotRequestor {
    next_id: Arc<AtomicUsize>,
    entries: moka::future::Cache<String, RequestorResponseSnapshot>,
    file: String,
    aggressive_save: bool,
    fail_on_miss: bool,
    ignore_snapshot: bool,
}

impl SnapshotRequestor {
    /// Creates a new instance by wrapping an existing [`HttpRequestor`] and capturing
    /// the request/response pairs.
    ///
    /// The constructor does not load any [snapshot entries](SnapshotRequestorLayer) from
    /// the `snapshot_file`.
    /// The [`SnapshotRequestorLayer::load`] method must be used after construction to do that.
    pub fn new(snapshot_file: &str) -> Self {
        Self {
            next_id: Arc::new(AtomicUsize::new(1)),
            entries: moka::future::Cache::builder().build(),
            file: snapshot_file.to_owned(),
            aggressive_save: false,
            fail_on_miss: false,
            ignore_snapshot: false,
        }
    }
    /// Gets the path to the snapshot disk file.
    pub fn snapshot_path(&self) -> &str {
        &self.file
    }

    /// Clears all entries from the snapshot in memory.
    /// The snapshot file is not changed.
    pub fn clear(&self) {
        self.entries.invalidate_all();
        self.next_id.store(1, Ordering::Relaxed);
    }

    /// Clears all entries and loads them from the snapshot file.
    /// If `fail_on_miss` is set and the data is successfully loaded, all later
    /// requests that miss the loaded snapshot will result in HTTP error 404.
    pub async fn try_load(&mut self, fail_on_miss: bool) -> Result<(), std::io::Error> {
        if self.ignore_snapshot {
            return Ok(());
        }

        let loaded = serde_yaml::from_reader::<_, Vec<RequestorResponseSnapshot>>(std::fs::File::open(&self.file)?)
            .map_err(std::io::Error::other)?;

        self.clear();

        let loaded_len = futures::stream::iter(loaded)
            .then(|entry| {
                self.next_id.fetch_max(entry.id, Ordering::Relaxed);
                self.entries.insert(entry.request.clone(), entry)
            })
            .collect::<Vec<_>>()
            .await
            .len();

        if loaded_len > 0 {
            self.fail_on_miss = fail_on_miss;
        }

        tracing::debug!("snapshot with {loaded_len} entries has been loaded from {}", &self.file);
        Ok(())
    }

    /// Similar as [`SnapshotRequestorLayer::try_load`], except that no entries are cleared if the load fails.
    ///
    /// This method consumes and returns self for easier call chaining.
    pub async fn load(mut self, fail_on_miss: bool) -> Self {
        let _ = self.try_load(fail_on_miss).await;
        self
    }

    /// Forces saving to disk on each newly inserted entry.
    ///
    /// Use this only when the expected number of entries in the snapshot is small.
    pub fn with_aggresive_save(mut self) -> Self {
        self.aggressive_save = true;
        self
    }

    /// If set, the snapshot data will be ignored and resolution
    /// will always be done with the inner requestor.
    ///
    /// This will inhibit any attempts to [`load`](SnapshotRequestorLayer::try_load) or
    /// [`save`](SnapshotRequestorLayer::save) snapshot data.
    pub fn with_ignore_snapshot(mut self, ignore_snapshot: bool) -> Self {
        self.ignore_snapshot = ignore_snapshot;
        self
    }

    /// Save the currently cached entries to the snapshot file on disk.
    ///
    /// Note that this method is automatically called on Drop, so usually it is unnecessary
    /// to call it explicitly.
    pub fn save(&self) -> Result<(), std::io::Error> {
        if self.ignore_snapshot {
            return Ok(());
        }

        let mut values: Vec<RequestorResponseSnapshot> = self.entries.iter().map(|(_, r)| r).collect();
        values.sort_unstable_by_key(|a| a.id);

        let mut writer = BufWriter::new(std::fs::File::create(&self.file)?);

        serde_yaml::to_writer(&mut writer, &values).map_err(std::io::Error::other)?;

        writer.flush()?;

        tracing::debug!("snapshot with {} entries saved to file {}", values.len(), self.file);
        Ok(())
    }
}

impl Drop for SnapshotRequestor {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            tracing::error!("failed to save snapshot: {e}");
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotRequestorLayer {
    snapshot_requestor: SnapshotRequestor,
}

impl SnapshotRequestorLayer {
    pub fn new(snapshot_file: &str) -> Self {
        Self {
            snapshot_requestor: SnapshotRequestor::new(snapshot_file),
        }
    }

    pub fn from_requestor(snapshot_requestor: SnapshotRequestor) -> Self {
        Self { snapshot_requestor }
    }
}
#[derive(Debug, Clone)]
pub struct SnapshotRequestorService<S> {
    inner: S,
    snapshot_requestor: SnapshotRequestor,
}

// Implement tower::Layer for MetricsLayer.
impl<S> Layer<S> for SnapshotRequestorLayer {
    type Service = SnapshotRequestorService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SnapshotRequestorService {
            inner,
            snapshot_requestor: self.snapshot_requestor.clone(), // next_id: self.next_id.clone(),
                                                                 // entries: self.entries.clone(),
                                                                 // file: self.file.clone(),
                                                                 // aggressive_save: self.aggressive_save.clone(),
                                                                 // fail_on_miss: self.fail_on_miss.clone(),
                                                                 // ignore_snapshot: self.ignore_snapshot.clone(),
        }
    }
}

/// Implement the [`tower::Service`] trait for the [`MetricsService`].
impl<S> Service<RequestPacket> for SnapshotRequestorService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError> + Send + 'static + Clone,
    S::Error: Send + 'static + Debug,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        let mut inner = self.inner.clone(); // Clone service
        let snapshot_requestor = self.snapshot_requestor.clone(); // Clone Arc or similar wrapper

        let future = inner.call(request.clone()); // Move request too

        Box::pin(async move {
            let res = future.await;

            let request_string = serde_json::to_string(&request)
                .map_err(|e| TransportErrorKind::Custom(format!("serialize error: {e}").into()))?;

            let inserted = AtomicBool::new(false);

            let result = snapshot_requestor
                .entries
                .entry(request_string.clone())
                .or_try_insert_with(async {
                    if snapshot_requestor.fail_on_miss {
                        tracing::error!("{request_string} is missing in {}", &snapshot_requestor.file);
                        return Err(TransportErrorKind::http_error(
                            http_types::StatusCode::NotFound.into(),
                            "".into(),
                        ));
                    }

                    let response_string = match &res {
                        Ok(result) => match result {
                            ResponsePacket::Single(resp) => match &resp.payload {
                                ResponsePayload::Success(success_payload) => success_payload.to_string(),
                                ResponsePayload::Failure(e) => {
                                    return Err(TransportErrorKind::Custom(format!("RPC error: {e}").into()).into());
                                }
                            },
                            ResponsePacket::Batch(batch) => {
                                let mut responses = Vec::with_capacity(batch.len());
                                for (i, resp) in batch.iter().enumerate() {
                                    match &resp.payload {
                                        ResponsePayload::Success(success_payload) => {
                                            responses.push(success_payload.to_string());
                                        }
                                        ResponsePayload::Failure(e) => {
                                            return Err(TransportErrorKind::Custom(
                                                format!("RPC error in batch item #{i}: {e}").into(),
                                            )
                                            .into());
                                        }
                                    }
                                }
                                responses.join(", ")
                            }
                        },
                        Err(err) => {
                            error!(error = ?err, "Error occurred while processing request");
                            return Err(TransportErrorKind::Custom(
                                format!("Error occurred while processing request: {err}").into(),
                            )
                            .into());
                        }
                    };

                    let id = snapshot_requestor.next_id.fetch_add(1, Ordering::SeqCst);
                    inserted.store(true, Ordering::Relaxed);
                    tracing::debug!("saved new snapshot entry #{id}");

                    Ok(RequestorResponseSnapshot {
                        id,
                        request: request_string.clone(),
                        response: response_string,
                    })
                })
                .await
                .map(|e| e.into_value().response.into_bytes().into_boxed_slice())
                .map_err(|e| TransportErrorKind::Custom(format!("{e}").into()))?;

            if inserted.load(Ordering::Relaxed) && snapshot_requestor.aggressive_save {
                tracing::debug!("{request_string} was NOT found and was resolved");
                snapshot_requestor
                    .save()
                    .map_err(|e| TransportErrorKind::Custom(format!("{e}").into()))?;
            } else {
                tracing::debug!("{request_string} was found");
            }

            res
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy::providers::Provider;
    use alloy::providers::ProviderBuilder;
    use alloy::rpc::client::ClientBuilder;
    use alloy::signers::local::PrivateKeySigner;
    use alloy::transports::layers::RetryBackoffLayer;
    use anyhow::Ok;
    use hopr_async_runtime::prelude::sleep;
    use hopr_chain_types::utils::create_anvil;
    use hopr_chain_types::{ContractAddresses, ContractInstances};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::primitives::Address;
    use serde_json::json;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    use crate::client::DefaultRetryPolicy;
    use crate::client::MetricsLayer;
    use crate::client::SnapshotRequestor;
    use crate::client::SnapshotRequestorLayer;
    use crate::client::ZeroRetryPolicy;
    use crate::transport::SurfTransport;

    #[tokio::test]
    async fn test_client_should_deploy_contracts_via_surf() -> anyhow::Result<()> {
        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let signer_chain_key = ChainKeypair::from_secret(signer.to_bytes().as_ref())?;

        let transport_client = SurfTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).on_client(rpc_client);

        let contracts = ContractInstances::deploy_for_testing(provider.clone(), &signer_chain_key)
            .await
            .expect("deploy failed");

        let contract_addrs = ContractAddresses::from(&contracts);

        assert_ne!(contract_addrs.token, Address::default());
        assert_ne!(contract_addrs.channels, Address::default());
        assert_ne!(contract_addrs.announcements, Address::default());
        assert_ne!(contract_addrs.network_registry, Address::default());
        assert_ne!(contract_addrs.safe_registry, Address::default());
        assert_ne!(contract_addrs.price_oracle, Address::default());

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_deploy_contracts_via_reqwest() -> anyhow::Result<()> {
        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let signer_chain_key = ChainKeypair::from_secret(signer.to_bytes().as_ref())?;

        let rpc_client = ClientBuilder::default().http(anvil.endpoint_url());

        let provider = ProviderBuilder::new().wallet(signer).on_client(rpc_client);

        let contracts = ContractInstances::deploy_for_testing(provider.clone(), &signer_chain_key)
            .await
            .expect("deploy failed");

        let contract_addrs = ContractAddresses::from(&contracts);

        assert_ne!(contract_addrs.token, Address::default());
        assert_ne!(contract_addrs.channels, Address::default());
        assert_ne!(contract_addrs.announcements, Address::default());
        assert_ne!(contract_addrs.network_registry, Address::default());
        assert_ne!(contract_addrs.safe_registry, Address::default());
        assert_ne!(contract_addrs.price_oracle, Address::default());

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_get_block_number() -> anyhow::Result<()> {
        let block_time = Duration::from_secs(1);

        let anvil = create_anvil(Some(block_time));
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        // let signer_chain_key = ChainKeypair::from_secret(signer.to_bytes().as_ref())?;

        let transport_client = SurfTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).on_client(rpc_client);

        let mut last_number = 0;

        for _ in 0..3 {
            sleep(block_time).await;

            // let number: ethers::types::U64 = client.request("eth_blockNumber", ()).await?;
            let num = provider.get_block_number().await?;

            assert!(num > last_number, "next block number must be greater");
            last_number = num;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_get_block_number_with_metrics_without_retry() -> anyhow::Result<()> {
        let block_time = Duration::from_secs(1);

        let anvil = create_anvil(Some(block_time));
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();

        let transport_client = SurfTransport::new(anvil.endpoint_url());

        // additional retry layer
        let retry_layer = RetryBackoffLayer::new(2, 100, 100);

        let rpc_client = ClientBuilder::default()
            .layer(retry_layer)
            .layer(MetricsLayer)
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).on_client(rpc_client);

        let mut last_number = 0;

        for _ in 0..3 {
            sleep(block_time).await;

            // let number: ethers::types::U64 = client.request("eth_blockNumber", ()).await?;
            let num = provider.get_block_number().await?;

            assert!(num > last_number, "next block number must be greater");
            last_number = num;
        }

        // FIXME: cannot get the private field `requests_enqueued`
        // assert_eq!(
        //     0,
        //     rpc_client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero on successful requests"
        // );

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_fail_on_malformed_request() -> anyhow::Result<()> {
        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();

        let transport_client = SurfTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber_bla".into(), ())
            .await
            .expect_err("expected error");

        assert!(matches!(err, alloy::transports::RpcError::ErrorResp(..)));

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_fail_on_malformed_response() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("}malformed{")
            .expect(1)
            .create();

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(
            err,
            alloy::transports::RpcError::DeserError { err: _, text: _ }
        ));
    }

    #[tokio::test]
    async fn test_client_should_retry_on_http_error() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(http_types::StatusCode::TooManyRequests as usize)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("{}")
            .expect(3)
            .create();

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // TODO: FIXME: implement a CustomRetryBackoff policy/service and test its `requests_enqueued`
        // let client = JsonRpcProviderClient::new(
        //     &server.url(),
        //     SurfRequestor::default(),
        //     SimpleJsonRpcRetryPolicy {
        //         max_retries: Some(2),
        //         retryable_http_errors: vec![http_types::StatusCode::TooManyRequests],
        //         initial_backoff: Duration::from_millis(100),
        //         ..SimpleJsonRpcRetryPolicy::default()
        //     },
        // );

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, alloy::transports::RpcError::Transport(..)));

        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[tokio::test]
    async fn test_client_should_not_retry_with_zero_retry_policy() {
        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("POST", "/")
            .with_status(404)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("{}")
            .expect(1)
            .create();

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(2, 100, 100, ZeroRetryPolicy))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, alloy::transports::RpcError::Transport(..)));
        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[tokio::test]
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

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let simple_json_rpc_retry_policy = DefaultRetryPolicy {
            initial_backoff: Duration::from_millis(100),
            retryable_json_rpc_errors: vec![-32603],
            ..DefaultRetryPolicy::default()
        };
        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(
                2,
                100,
                100,
                simple_json_rpc_retry_policy,
            ))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, alloy::transports::RpcError::Transport(..)));
        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[tokio::test]
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

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let simple_json_rpc_retry_policy = DefaultRetryPolicy {
            initial_backoff: Duration::from_millis(100),
            retryable_json_rpc_errors: vec![],
            ..DefaultRetryPolicy::default()
        };
        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(
                2,
                100,
                100,
                simple_json_rpc_retry_policy,
            ))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, alloy::transports::RpcError::ErrorResp(..)));

        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[tokio::test]
    async fn test_client_should_retry_on_nonretryable_json_rpc_error_if_min_retries_is_given() {
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
            .expect(2)
            .create();

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let simple_json_rpc_retry_policy = DefaultRetryPolicy {
            initial_backoff: Duration::from_millis(100),
            retryable_json_rpc_errors: vec![],
            min_retries: Some(1),
            ..DefaultRetryPolicy::default()
        };
        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(
                2,
                100,
                100,
                simple_json_rpc_retry_policy,
            ))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        // FIXME: implement minimum retry, and enable the assert
        // m.assert();
        assert!(matches!(err, alloy::transports::RpcError::ErrorResp(..)));

        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[tokio::test]
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

        let transport_client = SurfTransport::new(url::Url::parse(&server.url()).unwrap());

        let simple_json_rpc_retry_policy = DefaultRetryPolicy {
            initial_backoff: Duration::from_millis(100),
            retryable_json_rpc_errors: vec![-32600],
            min_retries: Some(1),
            ..DefaultRetryPolicy::default()
        };
        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(
                2,
                100,
                100,
                simple_json_rpc_retry_policy,
            ))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().on_client(rpc_client);

        let err = provider
            .raw_request::<(), alloy::primitives::U64>("eth_blockNumber".into(), ())
            .await
            .expect_err("expected error");

        m.assert();
        assert!(matches!(err, alloy::transports::RpcError::Transport(..)));

        // TODO: Create a customize RetryBackoffService that exposes `requests_enqueued`
        // assert_eq!(
        //     0,
        //     client.requests_enqueued.load(Ordering::SeqCst),
        //     "retry queue should be zero when policy says no more retries"
        // );
    }

    #[test_log::test(tokio::test)]
    async fn test_client_from_file() -> anyhow::Result<()> {
        let block_time = Duration::from_millis(1100);
        let snapshot_file = NamedTempFile::new()?;

        let anvil = create_anvil(Some(block_time));

        {
            let mut last_number = 0;

            let transport_client = SurfTransport::new(anvil.endpoint_url());

            let rpc_client = ClientBuilder::default()
                .layer(RetryBackoffLayer::new_with_policy(
                    2,
                    100,
                    100,
                    DefaultRetryPolicy::default(),
                ))
                .layer(SnapshotRequestorLayer::new(snapshot_file.path().to_str().unwrap()))
                .transport(transport_client.clone(), transport_client.guess_local());

            let provider = ProviderBuilder::new().on_client(rpc_client);

            for _ in 0..3 {
                sleep(block_time).await;

                // let number: ethers::types::U64 = client.request("eth_blockNumber", ()).await?;
                let num = provider.get_block_number().await?;

                assert!(num > last_number, "next block number must be greater");
                last_number = num;
            }
        }

        {
            let transport_client = SurfTransport::new(anvil.endpoint_url());

            let snapshot_requestor = SnapshotRequestor::new(snapshot_file.path().to_str().unwrap())
                .load(true)
                .await;

            let rpc_client = ClientBuilder::default()
                .layer(RetryBackoffLayer::new_with_policy(
                    2,
                    100,
                    100,
                    DefaultRetryPolicy::default(),
                ))
                .layer(SnapshotRequestorLayer::from_requestor(snapshot_requestor))
                .transport(transport_client.clone(), transport_client.guess_local());

            let provider = ProviderBuilder::new().on_client(rpc_client);

            let mut last_number = 0;
            for _ in 0..3 {
                sleep(block_time).await;

                let num = provider.get_block_number().await?;

                assert!(num > last_number, "next block number must be greater");
                last_number = num;
            }
        }

        Ok(())
    }
}
