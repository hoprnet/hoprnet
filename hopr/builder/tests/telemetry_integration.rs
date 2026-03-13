//! Integration tests verifying that `hopr-metrics` instruments are properly
//! collected by both an OTLP-compatible reader and the Prometheus text exporter.
//!
//! Each test file compiles into its own binary, so the global `OnceLock` inside
//! `hopr-metrics` is fresh and can be initialised with a custom provider that
//! includes an [`InMemoryMetricExporter`] (simulating an OTLP ingestor) alongside
//! the [`PrometheusExporter`] (used by the `/metrics` HTTP endpoint).

use std::sync::OnceLock;

use anyhow::Context;
use opentelemetry_prometheus_text_exporter::PrometheusExporter;
use opentelemetry_sdk::metrics::{
    InMemoryMetricExporter, PeriodicReader, SdkMeterProvider,
    data::{AggregatedMetrics, Metric, MetricData, ResourceMetrics, ScopeMetrics},
};

/// Shared test state: provider + in-memory exporter.
///
/// All tests in this binary share the same global `hopr-metrics` state,
/// so we initialise once and hand out references.
struct TestState {
    provider: SdkMeterProvider,
    otlp_exporter: InMemoryMetricExporter,
}

fn test_state() -> &'static TestState {
    static STATE: OnceLock<TestState> = OnceLock::new();
    STATE.get_or_init(|| {
        let otlp_exporter = InMemoryMetricExporter::default();
        let otlp_reader = PeriodicReader::builder(otlp_exporter.clone()).build();

        let prometheus_exporter = PrometheusExporter::builder()
            .without_counter_suffixes()
            .without_units()
            .without_target_info()
            .without_scope_info()
            .build();

        let provider = SdkMeterProvider::builder()
            .with_reader(otlp_reader)
            .with_reader(prometheus_exporter.clone())
            .build();

        assert!(
            hopr_metrics::init_with_provider(prometheus_exporter, provider.clone()),
            "global metric state must not be initialised yet"
        );

        TestState {
            provider,
            otlp_exporter,
        }
    })
}

/// Verifies that metrics created via the `hopr-metrics` API are delivered to
/// an OTLP-compatible reader (simulated by [`InMemoryMetricExporter`]) and
/// simultaneously available through the Prometheus text exporter.
///
/// This proves the unified `SdkMeterProvider` pattern used in `hoprd`
/// correctly feeds both export paths.
#[tokio::test]
async fn metrics_are_collected_by_otlp_reader_and_prometheus_exporter() -> anyhow::Result<()> {
    let state = test_state();

    // -- act: create instruments and record values -----------------------------

    let counter =
        hopr_metrics::SimpleCounter::new("it_counter", "integration test counter").context("SimpleCounter::new")?;
    counter.increment_by(42);

    let gauge = hopr_metrics::SimpleGauge::new("it_gauge", "integration test gauge").context("SimpleGauge::new")?;
    gauge.set(3.14);

    let histogram =
        hopr_metrics::SimpleHistogram::new("it_histogram", "integration test histogram", vec![1.0, 5.0, 10.0])
            .context("SimpleHistogram::new")?;
    histogram.observe(2.5);
    histogram.observe(7.0);

    let multi_counter =
        hopr_metrics::MultiCounter::new("it_multi_counter", "integration test multi counter", &["version"])
            .context("MultiCounter::new")?;
    multi_counter.increment_by(&["1.0.0"], 10);
    multi_counter.increment_by(&["2.0.0"], 5);

    let multi_gauge = hopr_metrics::MultiGauge::new("it_multi_gauge", "integration test multi gauge", &["kind"])
        .context("MultiGauge::new")?;
    multi_gauge.set(&["tcp"], 100.0);
    multi_gauge.set(&["udp"], 200.0);

    // -- assert: OTLP reader (InMemoryMetricExporter) received metrics ---------

    state
        .provider
        .force_flush()
        .context("force_flush on SdkMeterProvider")?;

    let resource_metrics: Vec<ResourceMetrics> = state
        .otlp_exporter
        .get_finished_metrics()
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("get_finished_metrics")?;

    let names: Vec<String> = resource_metrics
        .iter()
        .flat_map(|rm: &ResourceMetrics| rm.scope_metrics())
        .flat_map(|sm: &ScopeMetrics| sm.metrics())
        .map(|m: &Metric| m.name().to_string())
        .collect();

    assert!(
        names.contains(&"it_counter".to_string()),
        "OTLP reader must contain 'it_counter', got: {names:?}"
    );
    assert!(
        names.contains(&"it_gauge".to_string()),
        "OTLP reader must contain 'it_gauge', got: {names:?}"
    );
    assert!(
        names.contains(&"it_histogram".to_string()),
        "OTLP reader must contain 'it_histogram', got: {names:?}"
    );
    assert!(
        names.contains(&"it_multi_counter".to_string()),
        "OTLP reader must contain 'it_multi_counter', got: {names:?}"
    );
    assert!(
        names.contains(&"it_multi_gauge".to_string()),
        "OTLP reader must contain 'it_multi_gauge', got: {names:?}"
    );

    // -- assert: shadow state matches recorded values --------------------------

    assert_eq!(counter.get(), 42, "SimpleCounter shadow state");
    assert!((gauge.get() - 3.14).abs() < f64::EPSILON, "SimpleGauge shadow state");

    // -- assert: OTLP data points carry correct values -------------------------

    let all_metrics: Vec<&Metric> = resource_metrics
        .iter()
        .flat_map(|rm: &ResourceMetrics| rm.scope_metrics())
        .flat_map(|sm: &ScopeMetrics| sm.metrics())
        .collect();

    // Verify counter value via OTLP data
    let counter_metric = all_metrics
        .iter()
        .find(|m: &&&Metric| m.name() == "it_counter")
        .context("it_counter metric not found in OTLP data")?;

    if let AggregatedMetrics::U64(MetricData::Sum(sum)) = counter_metric.data() {
        let total: u64 = sum.data_points().map(|dp| dp.value()).sum();
        assert_eq!(total, 42, "OTLP counter value must be 42");
    } else {
        anyhow::bail!("it_counter: unexpected data type: {:?}", counter_metric.data());
    }

    // Verify histogram observation count via OTLP data
    let histogram_metric = all_metrics
        .iter()
        .find(|m: &&&Metric| m.name() == "it_histogram")
        .context("it_histogram metric not found in OTLP data")?;

    if let AggregatedMetrics::F64(MetricData::Histogram(hist)) = histogram_metric.data() {
        let count: u64 = hist.data_points().map(|dp| dp.count()).sum();
        assert_eq!(count, 2, "OTLP histogram must have 2 observations");
    } else {
        anyhow::bail!("it_histogram: unexpected data type: {:?}", histogram_metric.data());
    }

    Ok(())
}

/// Verifies that the Prometheus text exporter produces well-formed output
/// with `# HELP` and `# TYPE` lines for each metric.
#[tokio::test]
async fn prometheus_text_format_contains_expected_lines() -> anyhow::Result<()> {
    let _state = test_state();

    let gauge = hopr_metrics::SimpleGauge::new("it_prom_gauge", "prom format gauge").context("SimpleGauge::new")?;
    gauge.set(99.0);

    let text = hopr_metrics::gather_all_metrics()
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("gather_all_metrics")?;

    assert!(
        text.contains("# HELP it_prom_gauge"),
        "must contain HELP line, got:\n{text}"
    );
    assert!(
        text.contains("# TYPE it_prom_gauge"),
        "must contain TYPE line, got:\n{text}"
    );
    assert!(text.contains("it_prom_gauge"), "must contain gauge name, got:\n{text}");

    Ok(())
}

/// Verifies that both export paths work simultaneously: metrics recorded
/// through the `hopr-metrics` wrapper appear in both the OTLP in-memory
/// exporter and the Prometheus text output at the same time.
#[tokio::test]
async fn dual_export_paths_are_consistent() -> anyhow::Result<()> {
    let state = test_state();

    let counter =
        hopr_metrics::SimpleCounter::new("it_dual_counter", "dual export counter").context("SimpleCounter::new")?;
    counter.increment_by(7);

    // Flush so the OTLP reader picks up the data
    state
        .provider
        .force_flush()
        .context("force_flush on SdkMeterProvider")?;

    // Check OTLP path
    let resource_metrics: Vec<ResourceMetrics> = state
        .otlp_exporter
        .get_finished_metrics()
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let otlp_has_metric = resource_metrics
        .iter()
        .flat_map(|rm: &ResourceMetrics| rm.scope_metrics())
        .flat_map(|sm: &ScopeMetrics| sm.metrics())
        .any(|m: &Metric| m.name() == "it_dual_counter");

    assert!(otlp_has_metric, "OTLP reader must contain 'it_dual_counter'");

    // Check Prometheus text path
    let prom_text = hopr_metrics::gather_all_metrics()
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("gather_all_metrics")?;

    assert!(
        prom_text.contains("it_dual_counter"),
        "Prometheus text must contain 'it_dual_counter', got:\n{prom_text}"
    );

    Ok(())
}
