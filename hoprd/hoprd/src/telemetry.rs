use std::{str::FromStr, string::ToString, time::Duration};

use tracing::{info, warn};
use tracing_subscriber::prelude::*;
#[cfg(all(feature = "telemetry", feature = "prometheus"))]
use {
    hopr_metrics::{PrometheusMetric, PrometheusMetricFamily, PrometheusMetricType, gather_metric_families},
    opentelemetry::metrics::{ObservableCounter, ObservableGauge},
    std::{
        collections::HashSet,
        sync::{
            Arc, Mutex,
            mpsc::{self, Sender},
        },
        thread::{self, JoinHandle},
    },
};
#[cfg(feature = "telemetry")]
use {
    opentelemetry::{
        KeyValue,
        logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity},
        metrics::MeterProvider as _,
        trace::TracerProvider,
    },
    opentelemetry_otlp::WithExportConfig as _,
    opentelemetry_sdk::{
        logs::{SdkLogger, SdkLoggerProvider},
        metrics::SdkMeterProvider,
        trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    },
    tracing::field::{Field, Visit},
};

#[cfg(feature = "telemetry")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::Display)]
enum OtlpSignal {
    #[strum(serialize = "traces")]
    Traces,

    #[strum(serialize = "logs")]
    Logs,

    #[strum(serialize = "metrics")]
    Metrics,
}

#[cfg(feature = "telemetry")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::Display)]
enum OtlpTransport {
    #[strum(serialize = "grpc")]
    Grpc,

    #[strum(serialize = "http", serialize = "https")]
    Http,
}

#[cfg(feature = "telemetry")]
impl OtlpTransport {
    fn from_env() -> Self {
        match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            Ok(raw_url) => Self::from_str(raw_url.trim().split_once("://").map(|(scheme, _)| scheme).unwrap_or(""))
                .unwrap_or(Self::Grpc),
            Err(_) => Self::Grpc,
        }
    }
}

#[cfg(feature = "telemetry")]
#[derive(Debug)]
struct OtlpConfig {
    enabled: bool,
    service_name: String,
    transport: OtlpTransport,
    signals: Vec<OtlpSignal>,
}

#[cfg(feature = "telemetry")]
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
        let mut signals = Vec::new();

        if let Ok(raw_signals) = std::env::var("HOPRD_OTEL_SIGNALS") {
            for signal in raw_signals.split(',') {
                let signal = signal.trim();
                if signal.is_empty() {
                    continue;
                }
                match OtlpSignal::from_str(signal) {
                    Ok(parsed) if !signals.contains(&parsed) => signals.push(parsed),
                    Err(_) => {
                        warn!(otel_signal = %signal, "Invalid OpenTelemetry signal specified in HOPRD_OTEL_SIGNALS environment variable");
                    }
                    _ => {}
                }
            }
        } else {
            signals.push(OtlpSignal::Traces);
        }

        if signals.is_empty() {
            signals.push(OtlpSignal::Traces);
        }

        Self {
            enabled,
            service_name,
            transport,
            signals,
        }
    }

    fn has_signal(&self, signal: OtlpSignal) -> bool {
        self.signals.contains(&signal)
    }
}

#[cfg(feature = "telemetry")]
#[derive(Clone)]
struct OtelLogsLayer {
    logger: SdkLogger,
}

#[cfg(feature = "telemetry")]
impl OtelLogsLayer {
    fn new(logger: SdkLogger) -> Self {
        Self { logger }
    }
}

#[cfg(feature = "telemetry")]
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

        if let Some(body) = visitor.body.take() {
            record.set_body(body.into());
        }

        record.add_attribute("target", metadata.target().to_string());
        if let Some(module_path) = metadata.module_path() {
            record.add_attribute("module_path", module_path.to_string());
        }
        if let Some(file) = metadata.file() {
            record.add_attribute("file", file.to_string());
        }
        if let Some(line) = metadata.line() {
            record.add_attribute("line", i64::from(line));
        }
        if !visitor.attributes.is_empty() {
            record.add_attributes(visitor.attributes);
        }

        self.logger.emit(record);
    }
}

#[cfg(feature = "telemetry")]
#[derive(Default)]
struct TracingEventVisitor {
    body: Option<String>,
    attributes: Vec<(String, AnyValue)>,
    timestamp: Option<std::time::SystemTime>,
}

#[cfg(feature = "telemetry")]
trait UnixTimestampCandidate {
    fn as_unix_timestamp(&self) -> Option<u64>;
}

#[cfg(feature = "telemetry")]
impl UnixTimestampCandidate for u64 {
    fn as_unix_timestamp(&self) -> Option<u64> {
        Some(*self)
    }
}

#[cfg(feature = "telemetry")]
impl UnixTimestampCandidate for i64 {
    fn as_unix_timestamp(&self) -> Option<u64> {
        u64::try_from(*self).ok()
    }
}

#[cfg(feature = "telemetry")]
impl UnixTimestampCandidate for &str {
    fn as_unix_timestamp(&self) -> Option<u64> {
        self.parse::<u64>().ok()
    }
}

#[cfg(feature = "telemetry")]
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

    fn maybe_record_unix_timestamp<T>(&mut self, field: &Field, value: T)
    where
        T: UnixTimestampCandidate,
    {
        if field.name() == "timestamp" && self.timestamp.is_none() {
            self.timestamp = value.as_unix_timestamp().and_then(unix_timestamp_to_system_time);
        }
    }
}

#[cfg(feature = "telemetry")]
impl Visit for TracingEventVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.maybe_record_unix_timestamp(field, value);
        self.record_body_or_attribute(field, value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.maybe_record_unix_timestamp(field, value);
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
        self.maybe_record_unix_timestamp(field, value);
        self.record_body_or_attribute(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_body_or_attribute(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_body_or_attribute(field, format!("{value:?}"));
    }
}

#[cfg(feature = "telemetry")]
fn unix_timestamp_to_system_time(value: u64) -> Option<std::time::SystemTime> {
    const NS_PER_SEC: u64 = 1_000_000_000;
    const US_PER_SEC: u64 = 1_000_000;
    const MS_PER_SEC: u64 = 1_000;

    let (units_per_second, nanos_multiplier) = [
        (NS_PER_SEC, 1),
        (US_PER_SEC, NS_PER_SEC / US_PER_SEC),
        (MS_PER_SEC, NS_PER_SEC / MS_PER_SEC),
    ]
    .iter()
    .find(|&&(units, _)| units == 1 || value >= units * NS_PER_SEC)
    .copied()
    .unwrap_or((1, 0));

    let secs = value / units_per_second;
    let nanos = ((value % units_per_second) * nanos_multiplier) as u32;
    std::time::UNIX_EPOCH.checked_add(std::time::Duration::new(secs, nanos))
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
struct OtelPrometheusMetricBridge {
    _state: Arc<Mutex<OtelPrometheusMetricBridgeState>>,
    refresh_stop_sender: Option<Sender<()>>,
    refresh_thread: Option<JoinHandle<()>>,
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
#[derive(Default)]
struct OtelPrometheusMetricBridgeState {
    registered_families: HashSet<String>,
    f64_counters: Vec<ObservableCounter<f64>>,
    f64_gauges: Vec<ObservableGauge<f64>>,
    u64_counters: Vec<ObservableCounter<u64>>,
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
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

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn build_prometheus_metric_bridge(meter_provider: &SdkMeterProvider) -> OtelPrometheusMetricBridge {
    let meter = meter_provider.meter("hoprd_prometheus_bridge");
    let state = Arc::new(Mutex::new(OtelPrometheusMetricBridgeState::default()));

    if let Ok(mut state_guard) = state.lock() {
        sync_prometheus_metric_families(&meter, &mut state_guard);
    }

    let (refresh_stop_sender, refresh_stop_receiver) = mpsc::channel();
    let refresh_state = Arc::clone(&state);
    let refresh_meter = meter.clone();
    let refresh_thread = match thread::Builder::new()
        .name("hoprd-otel-metrics-bridge".to_string())
        .spawn(move || {
            loop {
                match refresh_stop_receiver.recv_timeout(Duration::from_secs(2)) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Ok(mut state_guard) = refresh_state.lock() {
                            sync_prometheus_metric_families(&refresh_meter, &mut state_guard);
                        }
                    }
                }
            }
        }) {
        Ok(thread) => Some(thread),
        Err(error) => {
            warn!(error = %error, "Failed to spawn Prometheus OTEL bridge refresh thread");
            None
        }
    };

    OtelPrometheusMetricBridge {
        _state: state,
        refresh_stop_sender: Some(refresh_stop_sender),
        refresh_thread,
    }
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn sync_prometheus_metric_families(meter: &opentelemetry::metrics::Meter, state: &mut OtelPrometheusMetricBridgeState) {
    for family in gather_metric_families() {
        let family_key = format!("{:?}:{}", family.get_field_type(), family.name());
        if !state.registered_families.insert(family_key) {
            continue;
        }
        register_prometheus_metric_family(meter, &family, state);
    }
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn register_prometheus_metric_family(
    meter: &opentelemetry::metrics::Meter,
    family: &PrometheusMetricFamily,
    state: &mut OtelPrometheusMetricBridgeState,
) {
    let metric_name = family.name().to_string();

    match family.get_field_type() {
        PrometheusMetricType::COUNTER => {
            let family_name = metric_name.clone();
            let counter = meter
                .f64_observable_counter(metric_name)
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name, PrometheusMetricType::COUNTER) {
                        for metric in family.get_metric() {
                            if let Some(counter) = metric.get_counter().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
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
            let gauge = meter
                .f64_observable_gauge(metric_name)
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name, PrometheusMetricType::GAUGE) {
                        for metric in family.get_metric() {
                            if let Some(gauge) = metric.get_gauge().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
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
            let histogram_count = meter
                .u64_observable_counter(format!("{metric_name}_count"))
                .with_callback(move |observer| {
                    if let Some(family) =
                        find_prometheus_family(&family_name_for_count, PrometheusMetricType::HISTOGRAM)
                    {
                        for metric in family.get_metric() {
                            if let Some(histogram) = metric.get_histogram().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
                                observer.observe(histogram.get_sample_count(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.u64_counters.push(histogram_count);

            let family_name_for_sum = metric_name.clone();
            let histogram_sum = meter
                .f64_observable_counter(format!("{metric_name}_sum"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_sum, PrometheusMetricType::HISTOGRAM)
                    {
                        for metric in family.get_metric() {
                            if let Some(histogram) = metric.get_histogram().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
                                observer.observe(histogram.get_sample_sum(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_counters.push(histogram_sum);

            let family_name_for_bucket = metric_name.clone();
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
            let summary_count = meter
                .u64_observable_counter(format!("{metric_name}_count"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_count, PrometheusMetricType::SUMMARY)
                    {
                        for metric in family.get_metric() {
                            if let Some(summary) = metric.get_summary().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
                                observer.observe(summary.sample_count(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.u64_counters.push(summary_count);

            let family_name_for_sum = metric_name.clone();
            let summary_sum = meter
                .f64_observable_counter(format!("{metric_name}_sum"))
                .with_callback(move |observer| {
                    if let Some(family) = find_prometheus_family(&family_name_for_sum, PrometheusMetricType::SUMMARY) {
                        for metric in family.get_metric() {
                            if let Some(summary) = metric.get_summary().as_ref() {
                                let attributes = prometheus_metric_attributes(metric, None);
                                observer.observe(summary.sample_sum(), &attributes);
                            }
                        }
                    }
                })
                .build();
            state.f64_counters.push(summary_sum);

            let family_name_for_quantile = metric_name.clone();
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

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn find_prometheus_family(metric_name: &str, metric_type: PrometheusMetricType) -> Option<PrometheusMetricFamily> {
    gather_metric_families()
        .into_iter()
        .find(|family| family.name() == metric_name && family.get_field_type() == metric_type)
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn prometheus_metric_attributes(metric: &PrometheusMetric, extra: Option<(&str, String)>) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(metric.get_label().len() + usize::from(extra.is_some()));
    for label in metric.get_label() {
        attributes.push(KeyValue::new(label.name().to_string(), label.value().to_string()));
    }
    if let Some((key, value)) = extra {
        attributes.push(KeyValue::new(key.to_string(), value));
    }
    attributes
}

#[derive(Default)]
pub(super) struct TelemetryHandles {
    #[cfg(feature = "telemetry")]
    tracer_provider: Option<SdkTracerProvider>,
    #[cfg(feature = "telemetry")]
    logger_provider: Option<SdkLoggerProvider>,
    #[cfg(feature = "telemetry")]
    meter_provider: Option<SdkMeterProvider>,
    #[cfg(all(feature = "telemetry", feature = "prometheus"))]
    metric_bridge: Option<OtelPrometheusMetricBridge>,
}

impl Drop for TelemetryHandles {
    fn drop(&mut self) {
        #[cfg(feature = "telemetry")]
        {
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
}

pub(super) fn init_logger() -> anyhow::Result<TelemetryHandles> {
    let env_filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => tracing_subscriber::filter::EnvFilter::new("info")
            .add_directive("libp2p_swarm=info".parse()?)
            .add_directive("libp2p_mplex=info".parse()?)
            .add_directive("libp2p_tcp=info".parse()?)
            .add_directive("libp2p_dns=info".parse()?)
            .add_directive("multistream_select=info".parse()?)
            .add_directive("isahc=error".parse()?)
            .add_directive("sea_orm=warn".parse()?)
            .add_directive("sqlx=warn".parse()?)
            .add_directive("hyper_util=warn".parse()?),
    };

    #[cfg(feature = "prof")]
    let registry = tracing_subscriber::Registry::default()
        .with(
            env_filter
                .add_directive("tokio=trace".parse()?)
                .add_directive("runtime=trace".parse()?),
        )
        .with(console_subscriber::spawn());

    #[cfg(not(feature = "prof"))]
    let registry = tracing_subscriber::Registry::default().with(env_filter);

    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let format = if std::env::var("HOPRD_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false)
    {
        format.json().boxed()
    } else {
        format.boxed()
    };

    let registry = registry.with(format);

    let mut telemetry_handles = TelemetryHandles::default();

    #[cfg(feature = "telemetry")]
    {
        let config = OtlpConfig::from_env();
        if config.enabled {
            let resource = opentelemetry_sdk::Resource::builder()
                .with_service_name(config.service_name.clone())
                .build();

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
                let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
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
                        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
                        .with_timeout(Duration::from_secs(5))
                        .build()?,
                };

                let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource.clone())
                    .build();
                let logger = logger_provider.logger(env!("CARGO_PKG_NAME"));
                telemetry_handles.logger_provider = Some(logger_provider);
                Some(OtelLogsLayer::new(logger))
            } else {
                None
            };

            if config.has_signal(OtlpSignal::Metrics) {
                #[cfg(feature = "prometheus")]
                {
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

                    let meter_provider = SdkMeterProvider::builder()
                        .with_periodic_exporter(exporter)
                        .with_resource(resource.clone())
                        .build();
                    opentelemetry::global::set_meter_provider(meter_provider.clone());
                    telemetry_handles.metric_bridge = Some(build_prometheus_metric_bridge(&meter_provider));
                    telemetry_handles.meter_provider = Some(meter_provider);
                }
                #[cfg(not(feature = "prometheus"))]
                {
                    warn!("OpenTelemetry metrics requested but the `prometheus` feature is disabled");
                }
            }

            let enabled_signals = config
                .signals
                .iter()
                .copied()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",");

            match (trace_layer, logs_layer) {
                (Some(trace_layer), Some(logs_layer)) => {
                    tracing::subscriber::set_global_default(registry.with(trace_layer).with(logs_layer))?
                }
                (Some(trace_layer), None) => tracing::subscriber::set_global_default(registry.with(trace_layer))?,
                (None, Some(logs_layer)) => tracing::subscriber::set_global_default(registry.with(logs_layer))?,
                (None, None) => tracing::subscriber::set_global_default(registry)?,
            }

            info!(
                otel_service_name = %config.service_name,
                otel_signals = %enabled_signals,
                otel_protocol = %config.transport.to_string(),
                "OpenTelemetry enabled"
            );
        } else {
            tracing::subscriber::set_global_default(registry)?
        }
    }

    #[cfg(not(feature = "telemetry"))]
    tracing::subscriber::set_global_default(registry)?;

    Ok(telemetry_handles)
}

#[cfg(all(test, feature = "telemetry"))]
mod telemetry_timestamp_tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::unix_timestamp_to_system_time;

    #[test]
    fn parses_seconds_timestamp() {
        let value = 1_700_000_000_u64;
        assert_eq!(
            unix_timestamp_to_system_time(value),
            Some(UNIX_EPOCH + Duration::new(1_700_000_000, 0))
        );
    }

    #[test]
    fn parses_milliseconds_timestamp() {
        let value = 1_700_000_000_123_u64;
        assert_eq!(
            unix_timestamp_to_system_time(value),
            Some(UNIX_EPOCH + Duration::new(1_700_000_000, 123_000_000))
        );
    }

    #[test]
    fn parses_microseconds_timestamp() {
        let value = 1_700_000_000_123_456_u64;
        assert_eq!(
            unix_timestamp_to_system_time(value),
            Some(UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_000))
        );
    }

    #[test]
    fn parses_nanoseconds_timestamp() {
        let value = 1_700_000_000_123_456_789_u64;
        assert_eq!(
            unix_timestamp_to_system_time(value),
            Some(UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_789))
        );
    }
}
