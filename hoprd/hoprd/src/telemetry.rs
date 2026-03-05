use std::{
    collections::HashMap,
    str::FromStr,
    string::ToString,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use opentelemetry::{
    Key, KeyValue,
    logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity},
    metrics::{MeterProvider as _, ObservableCounter, ObservableGauge},
    trace::TracerProvider,
};
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::{
    logs::{SdkLogger, SdkLoggerProvider},
    metrics::SdkMeterProvider,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use tracing::field::{Field, Visit};
use tracing_subscriber::prelude::*;
#[cfg(feature = "prometheus")]
use {
    hopr_metrics::{PrometheusMetric, PrometheusMetricFamily, PrometheusMetricType, gather_metric_families},
    std::{
        collections::HashSet,
        sync::mpsc::{self, Sender},
        thread::{self, JoinHandle},
    },
};

flagset::flags! {
    #[repr(u8)]
    #[derive(PartialOrd, Ord, strum::EnumString, strum::Display)]
    enum OtlpSignal: u8 {
        #[strum(serialize = "traces")]
        Traces = 0b0000_0001,

        #[strum(serialize = "logs")]
        Logs = 0b0000_0010,

        #[strum(serialize = "metrics")]
        Metrics = 0b0000_0100,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::Display)]
enum OtlpTransport {
    #[strum(serialize = "grpc")]
    Grpc,

    #[strum(serialize = "http", serialize = "https")]
    Http,
}

impl OtlpTransport {
    fn from_env() -> Self {
        match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            Ok(raw_url) => Self::from_str(raw_url.trim().split_once("://").map(|(scheme, _)| scheme).unwrap_or(""))
                .unwrap_or(Self::Grpc),
            Err(_) => Self::Grpc,
        }
    }
}

#[derive(Debug)]
struct OtlpConfig {
    enabled: bool,
    service_name: String,
    transport: OtlpTransport,
    signals: flagset::FlagSet<OtlpSignal>,
}

impl OtlpConfig {
    fn from_env() -> Self {
        let enabled = matches!(
            std::env::var("HOPRD_USE_OPENTELEMETRY")
                .ok()
                .map(|v| v.trim().to_ascii_lowercase())
                .as_deref(),
            Some("1" | "true" | "yes" | "on")
        );
        let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").into());
        let transport = OtlpTransport::from_env();
        let mut signals = flagset::FlagSet::empty();

        if let Ok(raw_signals) = std::env::var("HOPRD_OTEL_SIGNALS") {
            for signal in raw_signals.split(',') {
                let signal = signal.trim();
                if signal.is_empty() {
                    continue;
                }
                match OtlpSignal::from_str(signal) {
                    Ok(parsed) => signals |= parsed,
                    Err(_) => {
                        tracing::warn!(otel_signal = %signal, "Invalid OpenTelemetry signal specified in HOPRD_OTEL_SIGNALS environment variable");
                    }
                }
            }
        } else {
            signals |= OtlpSignal::Traces;
        }

        if signals.is_empty() {
            signals |= OtlpSignal::Traces;
        }

        Self {
            enabled,
            service_name,
            transport,
            signals,
        }
    }

    fn has_signal(&self, signal: OtlpSignal) -> bool {
        self.signals.contains(signal)
    }
}

#[derive(Clone, Debug)]
pub(super) struct NodeTelemetryIdentity {
    pub(super) node_address: String,
    pub(super) node_peer_id: String,
}

impl NodeTelemetryIdentity {
    fn resource_attributes(&self) -> [KeyValue; 2] {
        [
            KeyValue::new("node_address", self.node_address.clone()),
            KeyValue::new("node_peer_id", self.node_peer_id.clone()),
        ]
    }
}

#[derive(Clone)]
struct OtelLogsLayer {
    logger: SdkLogger,
    node_identity: NodeTelemetryIdentity,
}

impl OtelLogsLayer {
    fn new(logger: SdkLogger, node_identity: NodeTelemetryIdentity) -> Self {
        Self { logger, node_identity }
    }
}

impl<S> tracing_subscriber::Layer<S> for OtelLogsLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = TracingEventVisitor::default();
        event.record(&mut visitor);

        let mut record = self.logger.create_log_record();
        let event_timestamp = visitor.timestamp.unwrap_or(std::time::SystemTime::now());

        let (severity_number, severity_text) = match *metadata.level() {
            tracing::Level::ERROR => (Severity::Error, "ERROR"),
            tracing::Level::WARN => (Severity::Warn, "WARN"),
            tracing::Level::INFO => (Severity::Info, "INFO"),
            tracing::Level::DEBUG => (Severity::Debug, "DEBUG"),
            tracing::Level::TRACE => (Severity::Trace, "TRACE"),
        };

        record.set_timestamp(event_timestamp);
        record.set_observed_timestamp(event_timestamp);
        record.set_target(metadata.target().to_string());
        record.set_severity_number(severity_number);
        record.set_severity_text(severity_text);

        let mut body = HashMap::<Key, AnyValue>::new();
        if let Some(message) = visitor.body.take() {
            body.insert(Key::new("message"), AnyValue::String(message.into()));
        }
        body.insert(Key::new("level"), AnyValue::String(metadata.level().to_string().into()));
        body.insert(
            Key::new("target"),
            AnyValue::String(metadata.target().to_string().into()),
        );
        if let Some(module_path) = metadata.module_path() {
            body.insert(
                Key::new("module_path"),
                AnyValue::String(module_path.to_string().into()),
            );
            record.add_attribute("module_path", module_path.to_string());
        }
        if let Some(file) = metadata.file() {
            body.insert(Key::new("file"), AnyValue::String(file.to_string().into()));
            record.add_attribute("file", file.to_string());
        }
        if let Some(line) = metadata.line() {
            body.insert(Key::new("line"), AnyValue::Int(i64::from(line)));
            record.add_attribute("line", i64::from(line));
        }
        body.insert(
            Key::new("node_address"),
            AnyValue::String(self.node_identity.node_address.clone().into()),
        );
        body.insert(
            Key::new("node_peer_id"),
            AnyValue::String(self.node_identity.node_peer_id.clone().into()),
        );
        if !visitor.attributes.is_empty() {
            body.insert(
                Key::new("attributes"),
                AnyValue::Map(Box::new(
                    visitor
                        .attributes
                        .iter()
                        .map(|(key, value)| (Key::new(key.clone()), value.clone()))
                        .collect(),
                )),
            );
        }
        record.set_body(AnyValue::Map(Box::new(body)));

        record.add_attribute("target", metadata.target().to_string());
        record.add_attribute("node_address", self.node_identity.node_address.clone());
        record.add_attribute("node_peer_id", self.node_identity.node_peer_id.clone());
        if !visitor.attributes.is_empty() {
            record.add_attributes(visitor.attributes);
        }

        self.logger.emit(record);
    }
}

#[derive(Default)]
struct TracingEventVisitor {
    body: Option<String>,
    attributes: Vec<(String, AnyValue)>,
    timestamp: Option<std::time::SystemTime>,
}

impl TracingEventVisitor {
    fn record_body_or_attribute<V>(&mut self, field: &Field, value: V)
    where
        V: Into<AnyValue> + ToString,
    {
        if field.name() == "message" {
            self.body = Some(value.to_string());
        } else {
            self.attributes.push((field.name().to_string(), value.into()));
        }
    }

    fn maybe_record_unix_timestamp_millis(&mut self, field: &Field, value: u64) {
        if field.name() == "timestamp" && self.timestamp.is_none() {
            self.timestamp = std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_millis(value));
        }
    }
}

impl Visit for TracingEventVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        if let Ok(value) = u64::try_from(value) {
            self.maybe_record_unix_timestamp_millis(field, value);
        }
        self.record_body_or_attribute(field, value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.maybe_record_unix_timestamp_millis(field, value);
        if value <= i64::MAX as u64 {
            self.record_body_or_attribute(field, value as i64);
        } else {
            self.record_body_or_attribute(field, value.to_string());
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_body_or_attribute(field, value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_body_or_attribute(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_body_or_attribute(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_body_or_attribute(field, format!("{value:?}"));
    }
}

#[derive(Default)]
struct OtelSessionMetricBridge {
    u64_counters: Vec<ObservableCounter<u64>>,
    u64_gauges: Vec<ObservableGauge<u64>>,
    f64_gauges: Vec<ObservableGauge<f64>>,
}

#[derive(Default)]
struct SessionMetricCallbackCache {
    refreshed_at: Option<Instant>,
    values_by_name: HashMap<String, Vec<(hopr_lib::SessionId, hopr_lib::SessionMetricValue)>>,
}

const SESSION_METRIC_CACHE_TTL: Duration = Duration::from_millis(100);

fn session_metric_attributes(session_id: hopr_lib::SessionId, node_identity: &NodeTelemetryIdentity) -> [KeyValue; 3] {
    [
        KeyValue::new("session_id", session_id.to_string()),
        KeyValue::new("node_address", node_identity.node_address.clone()),
        KeyValue::new("node_peer_id", node_identity.node_peer_id.clone()),
    ]
}

fn refresh_session_metric_callback_cache(cache: &mut SessionMetricCallbackCache) {
    let now = Instant::now();
    if cache
        .refreshed_at
        .is_some_and(|refreshed_at| now.duration_since(refreshed_at) <= SESSION_METRIC_CACHE_TTL)
    {
        return;
    }

    let mut values_by_name: HashMap<String, Vec<(hopr_lib::SessionId, hopr_lib::SessionMetricValue)>> = HashMap::new();
    for snapshot in hopr_lib::session_telemetry_snapshots() {
        let session_id = snapshot.session_id;
        for sample in hopr_lib::session_snapshot_metric_samples(&snapshot) {
            values_by_name
                .entry(sample.name)
                .or_default()
                .push((session_id, sample.value));
        }
    }

    cache.values_by_name = values_by_name;
    cache.refreshed_at = Some(now);
}

fn get_cached_session_metric_values(
    cache: &Arc<Mutex<SessionMetricCallbackCache>>,
    metric_name: &str,
) -> Vec<(hopr_lib::SessionId, hopr_lib::SessionMetricValue)> {
    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };
    refresh_session_metric_callback_cache(&mut cache_guard);
    cache_guard.values_by_name.get(metric_name).cloned().unwrap_or_default()
}

fn build_session_u64_observable_gauge(
    meter: &opentelemetry::metrics::Meter,
    metric_name: String,
    cache: Arc<Mutex<SessionMetricCallbackCache>>,
    node_identity: NodeTelemetryIdentity,
) -> ObservableGauge<u64> {
    let callback_metric_name = metric_name.clone();
    meter
        .u64_observable_gauge(metric_name)
        .with_callback(move |observer| {
            for (session_id, metric_value) in get_cached_session_metric_values(&cache, &callback_metric_name) {
                if let hopr_lib::SessionMetricValue::U64(value) = metric_value {
                    let attributes = session_metric_attributes(session_id, &node_identity);
                    observer.observe(value, &attributes);
                }
            }
        })
        .build()
}

fn build_session_u64_observable_counter(
    meter: &opentelemetry::metrics::Meter,
    metric_name: String,
    cache: Arc<Mutex<SessionMetricCallbackCache>>,
    node_identity: NodeTelemetryIdentity,
) -> ObservableCounter<u64> {
    let callback_metric_name = metric_name.clone();
    meter
        .u64_observable_counter(metric_name)
        .with_callback(move |observer| {
            for (session_id, metric_value) in get_cached_session_metric_values(&cache, &callback_metric_name) {
                if let hopr_lib::SessionMetricValue::U64(value) = metric_value {
                    let attributes = session_metric_attributes(session_id, &node_identity);
                    observer.observe(value, &attributes);
                }
            }
        })
        .build()
}

fn build_session_f64_observable_gauge(
    meter: &opentelemetry::metrics::Meter,
    metric_name: String,
    cache: Arc<Mutex<SessionMetricCallbackCache>>,
    node_identity: NodeTelemetryIdentity,
) -> ObservableGauge<f64> {
    let callback_metric_name = metric_name.clone();
    meter
        .f64_observable_gauge(metric_name)
        .with_callback(move |observer| {
            for (session_id, metric_value) in get_cached_session_metric_values(&cache, &callback_metric_name) {
                if let hopr_lib::SessionMetricValue::F64(value) = metric_value {
                    let attributes = session_metric_attributes(session_id, &node_identity);
                    observer.observe(value, &attributes);
                }
            }
        })
        .build()
}

fn build_session_metric_bridge(
    meter_provider: &SdkMeterProvider,
    node_identity: NodeTelemetryIdentity,
) -> OtelSessionMetricBridge {
    let meter = meter_provider.meter("hoprd_session_snapshot_bridge");
    let mut session_metrics = OtelSessionMetricBridge::default();
    let cache = Arc::new(Mutex::new(SessionMetricCallbackCache::default()));

    for metric_definition in hopr_lib::session_snapshot_metric_definitions() {
        match metric_definition.kind {
            hopr_lib::SessionMetricKind::U64Gauge => {
                session_metrics.u64_gauges.push(build_session_u64_observable_gauge(
                    &meter,
                    metric_definition.name,
                    Arc::clone(&cache),
                    node_identity.clone(),
                ))
            }
            hopr_lib::SessionMetricKind::U64Counter => {
                session_metrics.u64_counters.push(build_session_u64_observable_counter(
                    &meter,
                    metric_definition.name,
                    Arc::clone(&cache),
                    node_identity.clone(),
                ))
            }
            hopr_lib::SessionMetricKind::F64Gauge => {
                session_metrics.f64_gauges.push(build_session_f64_observable_gauge(
                    &meter,
                    metric_definition.name,
                    Arc::clone(&cache),
                    node_identity.clone(),
                ))
            }
        }
    }

    session_metrics
}

#[cfg(feature = "prometheus")]
struct OtelPrometheusMetricBridge {
    _state: Arc<Mutex<OtelPrometheusMetricBridgeState>>,
    refresh_stop_sender: Option<Sender<()>>,
    refresh_thread: Option<JoinHandle<()>>,
}

#[cfg(feature = "prometheus")]
#[derive(Default)]
struct OtelPrometheusMetricBridgeState {
    registered_families: HashSet<String>,
    f64_counters: Vec<ObservableCounter<f64>>,
    f64_gauges: Vec<ObservableGauge<f64>>,
    u64_counters: Vec<ObservableCounter<u64>>,
}

#[cfg(feature = "prometheus")]
impl Drop for OtelPrometheusMetricBridge {
    fn drop(&mut self) {
        if let Some(sender) = self.refresh_stop_sender.take() {
            let _ = sender.send(());
        }
        if let Some(join_handle) = self.refresh_thread.take() {
            let _ = join_handle.join();
        }
    }
}

#[cfg(feature = "prometheus")]
fn build_prometheus_metric_bridge(
    meter_provider: &SdkMeterProvider,
    node_identity: NodeTelemetryIdentity,
) -> OtelPrometheusMetricBridge {
    let meter = meter_provider.meter("hoprd_prometheus_bridge");
    let state = Arc::new(Mutex::new(OtelPrometheusMetricBridgeState::default()));

    if let Ok(mut state_guard) = state.lock() {
        sync_prometheus_metric_families(&meter, &mut state_guard, &node_identity);
    }

    let (refresh_stop_sender, refresh_stop_receiver) = mpsc::channel();
    let refresh_state = Arc::clone(&state);
    let refresh_meter = meter.clone();
    let refresh_node_identity = node_identity.clone();
    let refresh_thread = match thread::Builder::new()
        .name("hoprd-otel-metrics-bridge".to_string())
        .spawn(move || {
            loop {
                match refresh_stop_receiver.recv_timeout(Duration::from_secs(2)) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Ok(mut state_guard) = refresh_state.lock() {
                            sync_prometheus_metric_families(&refresh_meter, &mut state_guard, &refresh_node_identity);
                        }
                    }
                }
            }
        }) {
        Ok(thread) => Some(thread),
        Err(error) => {
            tracing::warn!(error = %error, "Failed to spawn Prometheus OTEL bridge refresh thread");
            None
        }
    };

    OtelPrometheusMetricBridge {
        _state: state,
        refresh_stop_sender: Some(refresh_stop_sender),
        refresh_thread,
    }
}

#[cfg(feature = "prometheus")]
fn sync_prometheus_metric_families(
    meter: &opentelemetry::metrics::Meter,
    state: &mut OtelPrometheusMetricBridgeState,
    node_identity: &NodeTelemetryIdentity,
) {
    for family in gather_metric_families() {
        let family_key = format!("{:?}:{}", family.get_field_type(), family.name());
        if !state.registered_families.insert(family_key) {
            continue;
        }
        register_prometheus_metric_family(meter, &family, state, node_identity);
    }
}

#[cfg(feature = "prometheus")]
fn register_prometheus_metric_family(
    meter: &opentelemetry::metrics::Meter,
    family: &PrometheusMetricFamily,
    state: &mut OtelPrometheusMetricBridgeState,
    node_identity: &NodeTelemetryIdentity,
) {
    let metric_name = family.name().to_string();

    match family.get_field_type() {
        PrometheusMetricType::COUNTER => {
            let family_name = metric_name.clone();
            let metric_node_identity = node_identity.clone();
            let counter = meter
                .f64_observable_counter(metric_name)
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name, PrometheusMetricType::COUNTER) {
                        for metric in family.get_metric() {
                            if let Some(counter) = metric.get_counter().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None, &metric_node_identity);
                                observer.observe(counter.value(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_counters.push(counter);
        }
        PrometheusMetricType::GAUGE => {
            let family_name = metric_name.clone();
            let metric_node_identity = node_identity.clone();
            let gauge = meter
                .f64_observable_gauge(metric_name)
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name, PrometheusMetricType::GAUGE) {
                        for metric in family.get_metric() {
                            if let Some(gauge) = metric.get_gauge().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None, &metric_node_identity);
                                observer.observe(gauge.value(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_gauges.push(gauge);
        }
        PrometheusMetricType::HISTOGRAM => {
            let family_name_for_count = metric_name.clone();
            let metric_node_identity_for_count = node_identity.clone();
            let histogram_count = meter
                .u64_observable_counter(format!("{metric_name}_count"))
                .with_callback(move |observer| {
                    if let Some(family) =
                        find_prometheus_family(&family_name_for_count, PrometheusMetricType::HISTOGRAM)
                    {
                        for metric in family.get_metric() {
                            if let Some(histogram) = metric.get_histogram().as_ref() {
                                let attributes =
                                    prometheus_metric_attributes(metric, None, &metric_node_identity_for_count);
                                observer.observe(histogram.get_sample_count(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.u64_counters.push(histogram_count);

            let family_name_for_sum = metric_name.clone();
            let metric_node_identity_for_sum = node_identity.clone();
            let histogram_sum = meter
                .f64_observable_counter(format!("{metric_name}_sum"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_sum, PrometheusMetricType::HISTOGRAM)
                    {
                        for metric in family.get_metric() {
                            if let Some(histogram) = metric.get_histogram().as_ref() {
                                let attributes =
                                    prometheus_metric_attributes(metric, None, &metric_node_identity_for_sum);
                                observer.observe(histogram.get_sample_sum(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_counters.push(histogram_sum);

            let family_name_for_bucket = metric_name.clone();
            let metric_node_identity_for_bucket = node_identity.clone();
            let histogram_bucket = meter
                .u64_observable_counter(format!("{metric_name}_bucket"))
                .with_callback(move |observer| {
                    if let Some(family) =
                        find_prometheus_family(&family_name_for_bucket, PrometheusMetricType::HISTOGRAM)
                    {
                        for metric in family.get_metric() {
                            if let Some(histogram) = metric.get_histogram().as_ref() {
                                for bucket in histogram.get_bucket() {
                                    let attributes = prometheus_metric_attributes(
                                        metric,
                                        Some(("le", bucket.upper_bound().to_string())),
                                        &metric_node_identity_for_bucket,
                                    );
                                    observer.observe(bucket.cumulative_count(), &attributes);
                                }
                            }
                        }
                    }
                })
                .build();
            state.u64_counters.push(histogram_bucket);
        }
        PrometheusMetricType::SUMMARY => {
            let family_name_for_count = metric_name.clone();
            let metric_node_identity_for_count = node_identity.clone();
            let summary_count = meter
                .u64_observable_counter(format!("{metric_name}_count"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_count, PrometheusMetricType::SUMMARY)
                    {
                        for metric in family.get_metric() {
                            if let Some(summary) = metric.get_summary().as_ref() {
                                let attributes =
                                    prometheus_metric_attributes(metric, None, &metric_node_identity_for_count);
                                observer.observe(summary.sample_count(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.u64_counters.push(summary_count);

            let family_name_for_sum = metric_name.clone();
            let metric_node_identity_for_sum = node_identity.clone();
            let summary_sum = meter
                .f64_observable_counter(format!("{metric_name}_sum"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_sum, PrometheusMetricType::SUMMARY) {
                        for metric in family.get_metric() {
                            if let Some(summary) = metric.get_summary().as_ref() {
                                let attributes =
                                    prometheus_metric_attributes(metric, None, &metric_node_identity_for_sum);
                                observer.observe(summary.sample_sum(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_counters.push(summary_sum);

            let family_name_for_quantile = metric_name.clone();
            let metric_node_identity_for_quantile = node_identity.clone();
            let summary_quantile = meter
                .f64_observable_gauge(metric_name)
                .with_callback(move |observer| {
                    if let Some(family) =
                        find_prometheus_family(&family_name_for_quantile, PrometheusMetricType::SUMMARY)
                    {
                        for metric in family.get_metric() {
                            if let Some(summary) = metric.get_summary().as_ref() {
                                for quantile in summary.get_quantile() {
                                    let attributes = prometheus_metric_attributes(
                                        metric,
                                        Some(("quantile", quantile.quantile().to_string())),
                                        &metric_node_identity_for_quantile,
                                    );
                                    observer.observe(quantile.value(), &attributes);
                                }
                            }
                        }
                    }
                })
                .build();
            state.f64_gauges.push(summary_quantile);
        }
        _ => {}
    }
}

#[cfg(feature = "prometheus")]
fn find_prometheus_family(metric_name: &str, metric_type: PrometheusMetricType) -> Option<PrometheusMetricFamily> {
    gather_metric_families()
        .into_iter()
        .find(|family| family.name() == metric_name && family.get_field_type() == metric_type)
}

#[cfg(feature = "prometheus")]
fn prometheus_metric_attributes(
    metric: &PrometheusMetric,
    extra: Option<(&str, String)>,
    node_identity: &NodeTelemetryIdentity,
) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(metric.get_label().len() + usize::from(extra.is_some()) + 2);
    for label in metric.get_label() {
        attributes.push(KeyValue::new(label.name().to_string(), label.value().to_string()));
    }
    if let Some((key, value)) = extra {
        attributes.push(KeyValue::new(key.to_string(), value));
    }
    attributes.push(KeyValue::new("node_address", node_identity.node_address.clone()));
    attributes.push(KeyValue::new("node_peer_id", node_identity.node_peer_id.clone()));
    attributes
}

#[derive(Default)]
pub(super) struct TelemetryHandles {
    tracer_provider: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    session_metric_bridge: Option<OtelSessionMetricBridge>,
    #[cfg(feature = "prometheus")]
    metric_bridge: Option<OtelPrometheusMetricBridge>,
}

impl Drop for TelemetryHandles {
    fn drop(&mut self) {
        if let Some(tracer_provider) = self.tracer_provider.take() {
            let _ = tracer_provider.shutdown();
        }
        if let Some(logger_provider) = self.logger_provider.take() {
            let _ = logger_provider.shutdown();
        }
        if let Some(meter_provider) = self.meter_provider.take() {
            let _ = meter_provider.shutdown();
        }
    }
}

fn build_otel_resource(config: &OtlpConfig, node_identity: &NodeTelemetryIdentity) -> opentelemetry_sdk::Resource {
    opentelemetry_sdk::Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_attributes(node_identity.resource_attributes())
        .build()
}

fn enabled_signal_names(config: &OtlpConfig, signals: &[OtlpSignal]) -> String {
    signals
        .iter()
        .copied()
        .filter(|signal| config.signals.contains(*signal))
        .map(|signal| signal.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn init_logging(node_identity: NodeTelemetryIdentity) -> anyhow::Result<TelemetryHandles> {
    let mut telemetry_handles = TelemetryHandles::default();
    let registry = crate::telemetry_common::build_base_subscriber()?;
    let config = OtlpConfig::from_env();

    if config.enabled {
        let resource = build_otel_resource(&config, &node_identity);

        let trace_layer = if config.has_signal(OtlpSignal::Traces) {
            let exporter = match config.transport {
                OtlpTransport::Grpc => opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_protocol(opentelemetry_otlp::Protocol::Grpc)
                    .with_timeout(Duration::from_secs(5))
                    .build()?,
                OtlpTransport::Http => opentelemetry_otlp::SpanExporter::builder()
                    .with_http()
                    .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
                    .with_timeout(Duration::from_secs(5))
                    .build()?,
            };
            let batch_processor =
                opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::builder(
                    exporter,
                    opentelemetry_sdk::runtime::Tokio,
                )
                .build();
            let tracer_provider = SdkTracerProvider::builder()
                .with_span_processor(batch_processor)
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_resource(resource.clone())
                .build();
            let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));
            telemetry_handles.tracer_provider = Some(tracer_provider);
            Some(tracing_opentelemetry::layer().with_tracer(tracer))
        } else {
            None
        };

        let logs_layer = if config.has_signal(OtlpSignal::Logs) {
            let exporter = match config.transport {
                OtlpTransport::Grpc => opentelemetry_otlp::LogExporter::builder()
                    .with_tonic()
                    .with_protocol(opentelemetry_otlp::Protocol::Grpc)
                    .with_timeout(Duration::from_secs(5))
                    .build()?,
                OtlpTransport::Http => opentelemetry_otlp::LogExporter::builder()
                    .with_http()
                    .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
                    .with_timeout(Duration::from_secs(5))
                    .build()?,
            };

            let batch_processor =
                opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::builder(
                    exporter,
                    opentelemetry_sdk::runtime::Tokio,
                )
                .build();
            let logger_provider = SdkLoggerProvider::builder()
                .with_log_processor(batch_processor)
                .with_resource(resource.clone())
                .build();
            let logger = logger_provider.logger(env!("CARGO_PKG_NAME"));
            telemetry_handles.logger_provider = Some(logger_provider);
            Some(OtelLogsLayer::new(logger, node_identity.clone()))
        } else {
            None
        };
        let enabled_signals = enabled_signal_names(&config, &[OtlpSignal::Traces, OtlpSignal::Logs]);
        let metrics_requested = config.has_signal(OtlpSignal::Metrics);

        match (trace_layer, logs_layer) {
            (Some(trace_layer), Some(logs_layer)) => {
                tracing::subscriber::set_global_default(registry.with(trace_layer).with(logs_layer))?
            }
            (Some(trace_layer), None) => tracing::subscriber::set_global_default(registry.with(trace_layer))?,
            (None, Some(logs_layer)) => tracing::subscriber::set_global_default(registry.with(logs_layer))?,
            (None, None) => tracing::subscriber::set_global_default(registry)?,
        }

        tracing::info!(
            otel_service_name = %config.service_name,
            otel_signals = %enabled_signals,
            otel_metrics_deferred = metrics_requested,
            otel_protocol = %config.transport.to_string(),
            node_address = %node_identity.node_address,
            node_peer_id = %node_identity.node_peer_id,
            "OpenTelemetry initialized"
        );
    } else {
        tracing::subscriber::set_global_default(registry)?;
    }

    Ok(telemetry_handles)
}

pub(super) fn init_metrics(
    telemetry_handles: &mut TelemetryHandles,
    node_identity: NodeTelemetryIdentity,
) -> anyhow::Result<()> {
    if telemetry_handles.meter_provider.is_some() {
        return Ok(());
    }

    let config = OtlpConfig::from_env();
    if !config.enabled || !config.has_signal(OtlpSignal::Metrics) {
        return Ok(());
    }

    let resource = build_otel_resource(&config, &node_identity);
    let exporter = match config.transport {
        OtlpTransport::Grpc => opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_timeout(Duration::from_secs(5))
            .build()?,
        OtlpTransport::Http => opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
            .with_timeout(Duration::from_secs(5))
            .build()?,
    };

    let reader = opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    telemetry_handles.session_metric_bridge = Some(build_session_metric_bridge(&meter_provider, node_identity.clone()));
    #[cfg(feature = "prometheus")]
    {
        telemetry_handles.metric_bridge = Some(build_prometheus_metric_bridge(&meter_provider, node_identity.clone()));
    }
    telemetry_handles.meter_provider = Some(meter_provider);

    let enabled_signals = enabled_signal_names(&config, &[OtlpSignal::Metrics]);
    tracing::info!(
        otel_service_name = %config.service_name,
        otel_signals = %enabled_signals,
        otel_protocol = %config.transport.to_string(),
        node_address = %node_identity.node_address,
        node_peer_id = %node_identity.node_peer_id,
        "OpenTelemetry metrics initialized"
    );

    Ok(())
}

pub(super) fn init_telemetry(node_identity: NodeTelemetryIdentity) -> anyhow::Result<TelemetryHandles> {
    let mut telemetry_handles = init_logging(node_identity.clone())?;
    init_metrics(&mut telemetry_handles, node_identity)?;
    Ok(telemetry_handles)
}
