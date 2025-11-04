//! Due to the migration of the RPC client to the `alloy` crate, this module contains implementation
//! and parameters of client layers. The underlying HTTP transport layer is defined in `transport.rs`.
//!
//! Extended layers of RPC clients:
//! - Replace the legacy retry backoff layer with the default [`RetryBackoffService`]. However the backoff calculation
//!   still needs to be improved, as the number of retries is not passed to the `backoff_hint` method.
//! - Add Metrics Layer
//! - Add Snapshot Layer
//! - Use tokio runtime for most of the tests
//!
//! This module contains defalut gas estimation constants for EIP-1559 for Gnosis chain,
use std::{
    fmt::Debug,
    future::IntoFuture,
    io::{BufWriter, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

/// as GasOracleMiddleware middleware is migrated to GasFiller
use alloy::{eips::eip1559::Eip1559Estimation, providers::layers::AnvilProvider};
use alloy::{
    network::{EthereumWallet, Network, TransactionBuilder},
    primitives::utils::parse_units,
    providers::{
        Identity, Provider, RootProvider, SendableTx,
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, FillerControlFlow, GasFiller, JoinFill, NonceFiller, TxFiller,
            WalletFiller,
        },
    },
    rpc::json_rpc::{ErrorPayload, RequestPacket, ResponsePacket, ResponsePayload},
    transports::{HttpError, TransportError, TransportErrorKind, TransportFut, TransportResult, layers::RetryPolicy},
};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tower::{Layer, Service};
use tracing::{error, trace};
use url::Url;
use validator::Validate;

/// Gas estimation constants for EIP-1559 for Gnosis chain.
/// These values are used to estimate the gas price for transactions.
/// As GasOracleMiddleware is migrated to GasFiller, they are replaced with
/// default values.
pub const EIP1559_FEE_ESTIMATION_DEFAULT_MAX_FEE_GNOSIS: u128 = 3_000_000_000;
pub const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE_GNOSIS: u128 = 100_000_000;

use crate::{rpc::DEFAULT_GAS_ORACLE_URL, transport::HttpRequestor};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_RPC_CALLS: hopr_metrics::MultiCounter = hopr_metrics::MultiCounter::new(
        "hopr_rpc_call_count",
        "Number of Ethereum RPC calls over HTTP and their result",
        &["call", "result"]
    )
    .unwrap();
    static ref METRIC_RPC_CALLS_TIMING: hopr_metrics::MultiHistogram = hopr_metrics::MultiHistogram::new(
        "hopr_rpc_call_time_sec",
        "Timing of RPC calls over HTTP in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 7.0, 10.0],
        &["call"]
    )
    .unwrap();
    static ref METRIC_RETRIES_PER_RPC_CALL: hopr_metrics::MultiHistogram = hopr_metrics::MultiHistogram::new(
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
/// - `max_rate_limit_retries`: (u32) The maximum number of retries for rate limit errors. Different from the legacy
///   implementation, there is always an upper limit.
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
#[serde_as]
#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
pub struct DefaultRetryPolicy {
    /// Minimum number of retries of any error, regardless the error code.
    ///
    /// Default is 0.
    #[validate(range(min = 0))]
    #[default(Some(0))]
    pub min_retries: Option<u32>,

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
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[default(
        _code = "vec![http::StatusCode::TOO_MANY_REQUESTS,http::StatusCode::GATEWAY_TIMEOUT,\
                 http::StatusCode::SERVICE_UNAVAILABLE]"
    )]
    pub retryable_http_errors: Vec<http::StatusCode>,

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
        let status_code = match http::StatusCode::try_from(http_err.status) {
            Ok(status_code) => status_code,
            Err(_) => return false,
        };
        self.retryable_http_errors.contains(&status_code)
    }
}

impl RetryPolicy for DefaultRetryPolicy {
    fn should_retry(&self, err: &TransportError) -> bool {
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

    // TODO(#7140): original implementation requires input param of `num_retries`
    // next_backoff = initial_backoff * (1 + backoff_coefficient)^(num_retries - 1)
    fn backoff_hint(&self, _error: &alloy::transports::TransportError) -> Option<std::time::Duration> {
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

/// Generic [`GasOracle`] gas price categories.
#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GasCategory {
    SafeLow,
    #[default]
    Standard,
    Fast,
    Fastest,
}

/// Use the underlying gas tracker API of GnosisScan to populate the gas price.
/// It returns gas price in gwei.
/// It implements the `GasOracle` trait.
/// If no Oracle URL is given, it returns no values.
#[derive(Clone, Debug)]
pub struct GasOracleFiller<C> {
    client: C,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GasOracleResponse {
    pub status: String,
    pub message: String,
    pub result: GasOracleResponseResult,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct GasOracleResponseResult {
    pub last_block: String,
    pub safe_gas_price: String,
    pub propose_gas_price: String,
    pub fast_gas_price: String,
}

impl GasOracleResponse {
    #[inline]
    pub fn gas_from_category(&self, gas_category: GasCategory) -> String {
        self.result.gas_from_category(gas_category)
    }
}

impl GasOracleResponseResult {
    fn gas_from_category(&self, gas_category: GasCategory) -> String {
        match gas_category {
            GasCategory::SafeLow => self.safe_gas_price.clone(),
            GasCategory::Standard => self.propose_gas_price.clone(),
            GasCategory::Fast => self.fast_gas_price.clone(),
            GasCategory::Fastest => self.fast_gas_price.clone(),
        }
    }
}

impl<C> GasOracleFiller<C>
where
    C: HttpRequestor + Clone,
{
    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn new(client: C, url: Option<Url>) -> Self {
        Self {
            client,
            url: url.unwrap_or_else(|| Url::parse(DEFAULT_GAS_ORACLE_URL).unwrap()),
            gas_category: GasCategory::Standard,
        }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform a request to the gas price API and deserialize the response.
    pub async fn query(&self) -> Result<GasOracleResponse, TransportError> {
        let raw_value = self
            .client
            .http_get(self.url.as_str())
            .await
            .map_err(TransportErrorKind::custom)?;

        let parsed: GasOracleResponse = serde_json::from_slice(raw_value.as_ref()).map_err(|e| {
            error!(%e, "failed to deserialize gas price API response");
            TransportErrorKind::Custom("failed to deserialize gas price API response".into())
        })?;

        Ok(parsed)
    }

    async fn prepare_legacy<P, N>(&self, provider: &P, tx: &N::TransactionRequest) -> TransportResult<GasOracleFillable>
    where
        P: Provider<N>,
        N: Network,
    {
        let gas_limit_fut = tx.gas_limit().map_or_else(
            || provider.estimate_gas(tx.clone()).into_future().right_future(),
            |gas_limit| async move { Ok(gas_limit) }.left_future(),
        );

        let res_fut = self.query();

        // Run both futures concurrently
        let (gas_limit, res) = futures::try_join!(gas_limit_fut, res_fut)?;

        // // Await the future to get the gas limit
        // let gas_limit = gas_limit_fut.await?;

        // let res = self.query().await?;
        let gas_price_in_gwei = res.gas_from_category(self.gas_category);
        let gas_price = parse_units(&gas_price_in_gwei, "gwei")
            .map_err(|e| TransportErrorKind::custom_str(&format!("Failed to parse gwei from gas oracle: {e}")))?;
        let gas_price_in_128: u128 = gas_price
            .get_absolute()
            .try_into()
            .map_err(|_| TransportErrorKind::custom_str("Conversion overflow"))?;

        Ok(GasOracleFillable::Legacy {
            gas_limit,
            gas_price: tx.gas_price().unwrap_or(gas_price_in_128),
        })
    }

    async fn prepare_1559<P, N>(&self, provider: &P, tx: &N::TransactionRequest) -> TransportResult<GasOracleFillable>
    where
        P: Provider<N>,
        N: Network,
    {
        let gas_limit_fut = tx.gas_limit().map_or_else(
            || provider.estimate_gas(tx.clone()).into_future().right_future(),
            |gas_limit| async move { Ok(gas_limit) }.left_future(),
        );

        // Await the future to get the gas limit
        let gas_limit = gas_limit_fut.await?;

        Ok(GasOracleFillable::Eip1559 {
            gas_limit,
            estimate: self.estimate_eip1559_fees(),
        })
    }

    // returns hardcoded (max_fee_per_gas, max_priority_fee_per_gas)
    // Due to foundry is unable to estimate EIP-1559 fees for L2s https://github.com/foundry-rs/foundry/issues/5709,
    // a hardcoded value of (3 gwei, 0.1 gwei) for Gnosischain is returned.
    fn estimate_eip1559_fees(&self) -> Eip1559Estimation {
        Eip1559Estimation {
            max_fee_per_gas: EIP1559_FEE_ESTIMATION_DEFAULT_MAX_FEE_GNOSIS,
            max_priority_fee_per_gas: EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE_GNOSIS,
        }
    }
}

/// An enum over the different types of gas fillable.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasOracleFillable {
    Legacy {
        gas_limit: u64,
        gas_price: u128,
    },
    Eip1559 {
        gas_limit: u64,
        estimate: Eip1559Estimation,
    },
}

impl<N, C> TxFiller<N> for GasOracleFiller<C>
where
    N: Network,
    C: HttpRequestor + Clone,
{
    type Fillable = GasOracleFillable;

    fn status(&self, tx: &<N as Network>::TransactionRequest) -> FillerControlFlow {
        // legacy and eip2930 tx
        if tx.gas_price().is_some() && tx.gas_limit().is_some() {
            return FillerControlFlow::Finished;
        }

        // eip1559
        if tx.max_fee_per_gas().is_some() && tx.max_priority_fee_per_gas().is_some() && tx.gas_limit().is_some() {
            return FillerControlFlow::Finished;
        }

        FillerControlFlow::Ready
    }

    fn fill_sync(&self, _tx: &mut SendableTx<N>) {}

    async fn prepare<P>(&self, provider: &P, tx: &<N as Network>::TransactionRequest) -> TransportResult<Self::Fillable>
    where
        P: Provider<N>,
    {
        if tx.gas_price().is_some() {
            self.prepare_legacy(provider, tx).await
        } else {
            match self.prepare_1559(provider, tx).await {
                // fallback to legacy
                Ok(estimate) => Ok(estimate),
                Err(e) => Err(e),
            }
        }
    }

    async fn fill(&self, fillable: Self::Fillable, mut tx: SendableTx<N>) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            match fillable {
                GasOracleFillable::Legacy { gas_limit, gas_price } => {
                    builder.set_gas_limit(gas_limit);
                    builder.set_gas_price(gas_price);
                }
                GasOracleFillable::Eip1559 { gas_limit, estimate } => {
                    builder.set_gas_limit(gas_limit);
                    builder.set_max_fee_per_gas(estimate.max_fee_per_gas);
                    builder.set_max_priority_fee_per_gas(estimate.max_priority_fee_per_gas);
                }
            }
        };
        Ok(tx)
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
    type Error = TransportError;
    type Future = TransportFut<'static>;
    type Response = ResponsePacket;

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
                    method_names.iter().for_each(|_m| {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_RPC_CALLS.increment(&[_m, "failure"]);
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
            snapshot_requestor: self.snapshot_requestor.clone(),
        }
    }
}

/// Implement the [`tower::Service`] trait for the [`SnapshotRequestorService`].
impl<S> Service<RequestPacket> for SnapshotRequestorService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError> + Send + 'static + Clone,
    S::Error: Send + 'static + Debug,
{
    type Error = TransportError;
    type Future = TransportFut<'static>;
    type Response = ResponsePacket;

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

            let _result = snapshot_requestor
                .entries
                .entry(request_string.clone())
                .or_try_insert_with(async {
                    if snapshot_requestor.fail_on_miss {
                        tracing::error!("{request_string} is missing in {}", &snapshot_requestor.file);
                        return Err(TransportErrorKind::http_error(
                            http::StatusCode::NOT_FOUND.into(),
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

pub type AnvilRpcClient = FillProvider<
    JoinFill<
        JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
        WalletFiller<EthereumWallet>,
    >,
    AnvilProvider<RootProvider>,
>;
/// Used for testing. Creates RPC client to the local Anvil instance.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_rpc_client_to_anvil(
    anvil: &alloy::node_bindings::AnvilInstance,
    signer: &hopr_crypto_types::keypairs::ChainKeypair,
) -> Arc<AnvilRpcClient> {
    use alloy::{
        providers::{ProviderBuilder, layers::AnvilLayer},
        rpc::client::ClientBuilder,
        signers::local::PrivateKeySigner,
        transports::http::ReqwestTransport,
    };
    use hopr_crypto_types::keypairs::Keypair;

    let wallet = PrivateKeySigner::from_slice(signer.secret().as_ref()).expect("failed to construct wallet");

    let transport_client = ReqwestTransport::new(anvil.endpoint_url());

    let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

    let provider = ProviderBuilder::new()
        .layer(AnvilLayer::default())
        .wallet(wallet)
        .connect_client(rpc_client);

    Arc::new(provider)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy::{
        network::TransactionBuilder,
        primitives::{U256, address},
        providers::{
            Provider, ProviderBuilder,
            fillers::{BlobGasFiller, CachedNonceManager, ChainIdFiller, GasFiller, NonceFiller},
        },
        rpc::{client::ClientBuilder, types::TransactionRequest},
        signers::local::PrivateKeySigner,
        transports::{http::ReqwestTransport, layers::RetryBackoffLayer},
    };
    use anyhow::Ok;
    use hopr_async_runtime::prelude::sleep;
    use hopr_chain_types::{ContractAddresses, ContractInstances, utils::create_anvil};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::primitives::Address;
    use serde_json::json;
    use tempfile::NamedTempFile;

    use crate::client::{
        DefaultRetryPolicy, GasOracleFiller, MetricsLayer, SnapshotRequestor, SnapshotRequestorLayer, ZeroRetryPolicy,
    };

    #[tokio::test]
    async fn test_client_should_deploy_contracts_via_reqwest() -> anyhow::Result<()> {
        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let signer_chain_key = ChainKeypair::from_secret(signer.to_bytes().as_ref())?;

        let rpc_client = ClientBuilder::default().http(anvil.endpoint_url());

        let provider = ProviderBuilder::new().wallet(signer).connect_client(rpc_client);

        let contracts = ContractInstances::deploy_for_testing(provider.clone(), &signer_chain_key)
            .await
            .expect("deploy failed");

        let contract_addrs = ContractAddresses::from(&contracts);

        assert_ne!(contract_addrs.token, Address::default());
        assert_ne!(contract_addrs.channels, Address::default());
        assert_ne!(contract_addrs.announcements, Address::default());
        assert_ne!(contract_addrs.network_registry, Address::default());
        assert_ne!(contract_addrs.node_safe_registry, Address::default());
        assert_ne!(contract_addrs.ticket_price_oracle, Address::default());

        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_get_block_number() -> anyhow::Result<()> {
        let block_time = Duration::from_millis(1100);

        let anvil = create_anvil(Some(block_time));
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).connect_client(rpc_client);

        let mut last_number = 0;

        for _ in 0..3 {
            sleep(block_time).await;

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

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        // additional retry layer
        let retry_layer = RetryBackoffLayer::new(2, 100, 100);

        let rpc_client = ClientBuilder::default()
            .layer(retry_layer)
            .layer(MetricsLayer)
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).connect_client(rpc_client);

        let mut last_number = 0;

        for _ in 0..3 {
            sleep(block_time).await;

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

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(signer).connect_client(rpc_client);

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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let too_many_requests: u16 = http::StatusCode::TOO_MANY_REQUESTS.as_u16();

        let m = server
            .mock("POST", "/")
            .with_status(too_many_requests as usize)
            .match_body(mockito::Matcher::PartialJson(json!({"method": "eth_blockNumber"})))
            .with_body("{}")
            .expect(3)
            .create();

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

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

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(2, 100, 100, ZeroRetryPolicy))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

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

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

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

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let _m = server
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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

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

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

        let transport_client = ReqwestTransport::new(url::Url::parse(&server.url()).unwrap());

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

        let provider = ProviderBuilder::new().connect_client(rpc_client);

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

            let transport_client = ReqwestTransport::new(anvil.endpoint_url());

            let rpc_client = ClientBuilder::default()
                .layer(RetryBackoffLayer::new_with_policy(
                    2,
                    100,
                    100,
                    DefaultRetryPolicy::default(),
                ))
                .layer(SnapshotRequestorLayer::new(snapshot_file.path().to_str().unwrap()))
                .transport(transport_client.clone(), transport_client.guess_local());

            let provider = ProviderBuilder::new().connect_client(rpc_client);

            for _ in 0..3 {
                sleep(block_time).await;

                let num = provider.get_block_number().await?;

                assert!(num > last_number, "next block number must be greater");
                last_number = num;
            }
        }

        {
            let transport_client = ReqwestTransport::new(anvil.endpoint_url());

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

            let provider = ProviderBuilder::new().connect_client(rpc_client);

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

    #[tokio::test]
    async fn test_client_should_call_on_gas_oracle_for_eip1559_tx() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("GET", "/gasapi.ashx?apikey=key&method=gasoracle")
            .with_status(http::StatusCode::ACCEPTED.as_u16().into())
            .with_body(r#"{"status":"1","message":"OK","result":{"LastBlock":"39864926","SafeGasPrice":"1.1","ProposeGasPrice":"1.1","FastGasPrice":"1.6","UsdPrice":"0.999968207972734"}}"#)
            .expect(0)
            .create();

        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());
        // let underlying_transport_client = transport_client.client().clone();

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .wallet(signer)
            .filler(NonceFiller::new(CachedNonceManager::default()))
            .filler(GasOracleFiller::new(
                transport_client.client().clone(),
                Some((server.url() + "/gasapi.ashx?apikey=key&method=gasoracle").parse()?),
            ))
            .filler(GasFiller)
            .connect_client(rpc_client);

        let tx = TransactionRequest::default()
            .with_chain_id(provider.get_chain_id().await?)
            .to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
            .value(U256::from(100))
            .transaction_type(2);

        let receipt = provider.send_transaction(tx).await?.get_receipt().await?;

        m.assert();
        assert_eq!(receipt.gas_used, 21000);
        Ok(())
    }

    #[tokio::test]
    async fn test_client_should_call_on_gas_oracle_for_legacy_tx() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut server = mockito::Server::new_async().await;

        let m = server
            .mock("GET", "/gasapi.ashx?apikey=key&method=gasoracle")
            .with_status(http::StatusCode::ACCEPTED.as_u16().into())
            .with_body(r#"{"status":"1","message":"OK","result":{"LastBlock":"39864926","SafeGasPrice":"1.1","ProposeGasPrice":"3.5","FastGasPrice":"1.6","UsdPrice":"0.999968207972734"}}"#)
            .expect(1)
            .create();

        let anvil = create_anvil(None);
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .wallet(signer)
            .filler(ChainIdFiller::default())
            .filler(NonceFiller::new(CachedNonceManager::default()))
            .filler(GasOracleFiller::new(
                transport_client.client().clone(),
                Some((server.url() + "/gasapi.ashx?apikey=key&method=gasoracle").parse()?),
            ))
            .filler(GasFiller)
            .filler(BlobGasFiller)
            .connect_client(rpc_client);

        // GasEstimationLayer requires chain_id to be set to handle EIP-1559 tx
        let tx = TransactionRequest::default()
            .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
            .with_value(U256::from(100))
            .with_gas_price(1000000000);

        let receipt = provider.send_transaction(tx).await?.get_receipt().await?;

        m.assert();
        assert_eq!(receipt.gas_used, 21000);
        Ok(())
    }
}
