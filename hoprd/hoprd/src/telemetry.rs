use std::time::Duration;

use tracing::{info, warn};
use tracing_subscriber::prelude::*;
#[cfg(all(feature = "telemetry", feature = "prometheus"))]
use {
    hopr_metrics::{PrometheusMetric, PrometheusMetricType, gather_metric_families},
    opentelemetry::metrics::{ObservableCounter, ObservableGauge},
};
#[cfg(feature = "telemetry")]
use {
    opentelemetry::{
        Context, KeyValue,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OtlpSignal {
    Traces,
    Logs,
    Metrics,
}

#[cfg(feature = "telemetry")]
impl OtlpSignal {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "traces" | "trace" => Some(Self::Traces),
            "logs" | "log" => Some(Self::Logs),
            "metrics" | "metric" => Some(Self::Metrics),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Traces => "traces",
            Self::Logs => "logs",
            Self::Metrics => "metrics",
        }
    }
}

#[cfg(feature = "telemetry")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OtlpTransport {
    Grpc,
    Http,
}

#[cfg(feature = "telemetry")]
impl OtlpTransport {
    fn from_env() -> (Self, Option<String>) {
        match std::env::var("HOPRD_OTEL_PROTOCOL")
            .ok()
            .map(|v| v.trim().to_ascii_lowercase())
        {
            None => (Self::Grpc, None),
            Some(value) if value == "grpc" => (Self::Grpc, None),
            Some(value) if value == "http" || value == "httpbinary" || value == "http-proto" => (Self::Http, None),
            Some(value) => (Self::Grpc, Some(value)),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Grpc => "grpc",
            Self::Http => "http",
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
    invalid_signals: Vec<String>,
    invalid_transport: Option<String>,
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
        let (transport, invalid_transport) = OtlpTransport::from_env();
        let mut signals = Vec::new();
        let mut invalid_signals = Vec::new();

        if let Ok(raw_signals) = std::env::var("HOPRD_OTEL_SIGNALS") {
            for signal in raw_signals.split(',') {
                let signal = signal.trim();
                if signal.is_empty() {
                    continue;
                }
                match OtlpSignal::from_str(signal) {
                    Some(parsed) if !signals.contains(&parsed) => signals.push(parsed),
                    Some(_) => {}
                    None => invalid_signals.push(signal.to_string()),
                }
            }
        }

        if signals.is_empty() {
            signals.push(OtlpSignal::Traces);
        }

        Self {
            enabled,
            service_name,
            transport,
            signals,
            invalid_signals,
            invalid_transport,
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
        let _suppress = Context::enter_telemetry_suppressed_scope();
        let metadata = event.metadata();
        let mut visitor = TracingEventVisitor::default();
        event.record(&mut visitor);

        let mut record = self.logger.create_log_record();
        let now = std::time::SystemTime::now();
        let (severity_number, severity_text) = severity_for_level(*metadata.level());

        record.set_timestamp(now);
        record.set_observed_timestamp(now);
        record.set_target(metadata.target().to_string());
        record.set_severity_number(severity_number);
        record.set_severity_text(severity_text);
        if let Some(body) = visitor.body.take() {
            record.set_body(body.into());
        } else {
            record.set_body(metadata.target().to_string().into());
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
}

#[cfg(feature = "telemetry")]
impl Visit for TracingEventVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_body_or_attribute(field, value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
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

#[cfg(feature = "telemetry")]
fn severity_for_level(level: tracing::Level) -> (Severity, &'static str) {
    if level == tracing::Level::ERROR {
        (Severity::Error, "ERROR")
    } else if level == tracing::Level::WARN {
        (Severity::Warn, "WARN")
    } else if level == tracing::Level::INFO {
        (Severity::Info, "INFO")
    } else if level == tracing::Level::DEBUG {
        (Severity::Debug, "DEBUG")
    } else {
        (Severity::Trace, "TRACE")
    }
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
struct OtelPrometheusMetricBridge {
    _counter: ObservableCounter<f64>,
    _gauge: ObservableGauge<f64>,
    _histogram_count: ObservableCounter<u64>,
    _histogram_sum: ObservableCounter<f64>,
    _histogram_bucket: ObservableCounter<u64>,
    _summary_count: ObservableCounter<u64>,
    _summary_sum: ObservableCounter<f64>,
    _summary_quantile: ObservableGauge<f64>,
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn build_prometheus_metric_bridge(meter_provider: &SdkMeterProvider) -> OtelPrometheusMetricBridge {
    let meter = meter_provider.meter("hoprd_prometheus_bridge");

    let counter = meter
        .f64_observable_counter("hoprd_prometheus_counter")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::COUNTER {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(counter) = metric.get_counter().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(counter.value(), &attributes);
                    }
                }
            }
        })
        .build();

    let gauge = meter
        .f64_observable_gauge("hoprd_prometheus_gauge")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::GAUGE {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(gauge) = metric.get_gauge().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(gauge.value(), &attributes);
                    }
                }
            }
        })
        .build();

    let histogram_count = meter
        .u64_observable_counter("hoprd_prometheus_histogram_count")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::HISTOGRAM {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(histogram) = metric.get_histogram().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(histogram.get_sample_count(), &attributes);
                    }
                }
            }
        })
        .build();

    let histogram_sum = meter
        .f64_observable_counter("hoprd_prometheus_histogram_sum")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::HISTOGRAM {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(histogram) = metric.get_histogram().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(histogram.get_sample_sum(), &attributes);
                    }
                }
            }
        })
        .build();

    let histogram_bucket = meter
        .u64_observable_counter("hoprd_prometheus_histogram_bucket")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::HISTOGRAM {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(histogram) = metric.get_histogram().as_ref() {
                        for bucket in histogram.get_bucket() {
                            let attributes = prometheus_metric_attributes(
                                &metric_name,
                                metric,
                                Some(("prom_bucket_le", bucket.upper_bound().to_string())),
                            );
                            observer.observe(bucket.cumulative_count(), &attributes);
                        }
                    }
                }
            }
        })
        .build();

    let summary_count = meter
        .u64_observable_counter("hoprd_prometheus_summary_count")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::SUMMARY {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(summary) = metric.get_summary().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(summary.sample_count(), &attributes);
                    }
                }
            }
        })
        .build();

    let summary_sum = meter
        .f64_observable_counter("hoprd_prometheus_summary_sum")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::SUMMARY {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(summary) = metric.get_summary().as_ref() {
                        let attributes = prometheus_metric_attributes(&metric_name, metric, None);
                        observer.observe(summary.sample_sum(), &attributes);
                    }
                }
            }
        })
        .build();

    let summary_quantile = meter
        .f64_observable_gauge("hoprd_prometheus_summary_quantile")
        .with_callback(|observer| {
            for family in gather_metric_families() {
                if family.get_field_type() != PrometheusMetricType::SUMMARY {
                    continue;
                }
                let metric_name = family.name().to_string();
                for metric in family.get_metric() {
                    if let Some(summary) = metric.get_summary().as_ref() {
                        for quantile in summary.get_quantile() {
                            let attributes = prometheus_metric_attributes(
                                &metric_name,
                                metric,
                                Some(("prom_summary_quantile", quantile.quantile().to_string())),
                            );
                            observer.observe(quantile.value(), &attributes);
                        }
                    }
                }
            }
        })
        .build();

    OtelPrometheusMetricBridge {
        _counter: counter,
        _gauge: gauge,
        _histogram_count: histogram_count,
        _histogram_sum: histogram_sum,
        _histogram_bucket: histogram_bucket,
        _summary_count: summary_count,
        _summary_sum: summary_sum,
        _summary_quantile: summary_quantile,
    }
}

#[cfg(all(feature = "telemetry", feature = "prometheus"))]
fn prometheus_metric_attributes(
    metric_name: &str,
    metric: &PrometheusMetric,
    extra: Option<(&str, String)>,
) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(metric.get_label().len() + 2);
    attributes.push(KeyValue::new("prom_metric_name", metric_name.to_string()));
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

            if let Some(invalid_transport) = &config.invalid_transport {
                warn!(
                    invalid_transport,
                    "invalid HOPRD_OTEL_PROTOCOL value, defaulting to grpc"
                );
            }
            if !config.invalid_signals.is_empty() {
                warn!(
                    invalid_signals = config.invalid_signals.join(","),
                    "ignoring invalid HOPRD_OTEL_SIGNALS values"
                );
            }

            let enabled_signals = config
                .signals
                .iter()
                .copied()
                .map(OtlpSignal::as_str)
                .collect::<Vec<_>>()
                .join(",");
            info!(
                otel_service_name = %config.service_name,
                otel_signals = %enabled_signals,
                otel_protocol = %config.transport.as_str(),
                "OpenTelemetry enabled"
            );

            match (trace_layer, logs_layer) {
                (Some(trace_layer), Some(logs_layer)) => {
                    tracing::subscriber::set_global_default(registry.with(trace_layer).with(logs_layer))?
                }
                (Some(trace_layer), None) => tracing::subscriber::set_global_default(registry.with(trace_layer))?,
                (None, Some(logs_layer)) => tracing::subscriber::set_global_default(registry.with(logs_layer))?,
                (None, None) => tracing::subscriber::set_global_default(registry)?,
            }
        } else {
            tracing::subscriber::set_global_default(registry)?
        }
    }

    #[cfg(not(feature = "telemetry"))]
    tracing::subscriber::set_global_default(registry)?;

    Ok(telemetry_handles)
}
