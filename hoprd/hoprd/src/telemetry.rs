use std::{str::FromStr, string::ToString, time::Duration};

use opentelemetry::{
    logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity},
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

#[derive(Clone)]
struct OtelLogsLayer {
    logger: SdkLogger,
}

impl OtelLogsLayer {
    fn new(logger: SdkLogger) -> Self {
        Self { logger }
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
pub(super) struct TelemetryHandles {
    tracer_provider: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    meter_provider: Option<SdkMeterProvider>,
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

pub(super) fn init_logger() -> anyhow::Result<TelemetryHandles> {
    let mut telemetry_handles = TelemetryHandles::default();
    let registry = crate::telemetry_common::build_base_subscriber()?;
    let config = OtlpConfig::from_env();

    // Build the Prometheus text exporter for the /metrics endpoint.
    // This is always created so that hopr-metrics instruments are collected
    // regardless of whether OTLP export is enabled.
    let prometheus_exporter = opentelemetry_prometheus_text_exporter::PrometheusExporter::builder()
        .without_counter_suffixes()
        .without_units()
        .without_target_info()
        .without_scope_info()
        .build();

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
                    .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
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
            Some(OtelLogsLayer::new(logger))
        } else {
            None
        };

        if config.has_signal(OtlpSignal::Metrics) {
            let otlp_exporter = match config.transport {
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

            let otlp_reader = opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder(
                otlp_exporter,
                opentelemetry_sdk::runtime::Tokio,
            )
            .build();

            // Build unified meter provider with both OTLP and Prometheus text readers
            let meter_provider = SdkMeterProvider::builder()
                .with_reader(otlp_reader)
                .with_reader(prometheus_exporter.clone())
                .with_resource(resource.clone())
                .build();
            opentelemetry::global::set_meter_provider(meter_provider.clone());

            // Initialize hopr-metrics with this unified provider so all instruments
            // feed into both OTLP and the Prometheus text endpoint
            if !hopr_metrics::init_with_provider(prometheus_exporter, meter_provider.clone()) {
                tracing::warn!("hopr-metrics global state was already initialized; custom provider not applied");
            }

            telemetry_handles.meter_provider = Some(meter_provider);
        } else {
            // OTLP metrics not requested, but still set up the Prometheus text exporter
            let meter_provider = SdkMeterProvider::builder()
                .with_reader(prometheus_exporter.clone())
                .with_resource(resource.clone())
                .build();
            if !hopr_metrics::init_with_provider(prometheus_exporter, meter_provider.clone()) {
                tracing::warn!("hopr-metrics global state was already initialized; custom provider not applied");
            }
            telemetry_handles.meter_provider = Some(meter_provider);
        }

        let enabled_signals = [OtlpSignal::Traces, OtlpSignal::Logs, OtlpSignal::Metrics]
            .into_iter()
            .filter(|signal| config.signals.contains(*signal))
            .map(|signal| signal.to_string())
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

        tracing::info!(
            otel_service_name = %config.service_name,
            otel_signals = %enabled_signals,
            otel_protocol = %config.transport.to_string(),
            "OpenTelemetry enabled"
        );
    } else {
        // OTEL disabled — still set up Prometheus text exporter for /metrics
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(prometheus_exporter.clone())
            .build();
        if !hopr_metrics::init_with_provider(prometheus_exporter, meter_provider.clone()) {
            tracing::warn!("hopr-metrics global state was already initialized; custom provider not applied");
        }
        telemetry_handles.meter_provider = Some(meter_provider);

        tracing::subscriber::set_global_default(registry)?;
    }

    Ok(telemetry_handles)
}
