use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};

use opentelemetry::{
    KeyValue,
    metrics::{Counter, Histogram, MeterProvider as _, UpDownCounter},
};
use opentelemetry_sdk::metrics::SdkMeterProvider;

/// Error type for metric operations.
#[derive(Debug)]
pub struct MetricError(String);

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MetricError {}

/// Result type for metric operations.
pub type MetricResult<T> = Result<T, MetricError>;

// ---------------------------------------------------------------------------
// Global metric state
// ---------------------------------------------------------------------------

struct GlobalMetricState {
    exporter: opentelemetry_prometheus_text_exporter::PrometheusExporter,
    provider: SdkMeterProvider,
}

static GLOBAL_STATE: OnceLock<GlobalMetricState> = OnceLock::new();

fn global_state() -> &'static GlobalMetricState {
    GLOBAL_STATE.get_or_init(|| {
        let exporter = opentelemetry_prometheus_text_exporter::PrometheusExporter::builder()
            .without_counter_suffixes()
            .without_units()
            .without_target_info()
            .without_scope_info()
            .build();
        let provider = SdkMeterProvider::builder().with_reader(exporter.clone()).build();
        GlobalMetricState { exporter, provider }
    })
}

/// Initializes the global metric state with the given exporter and provider.
///
/// This must be called **before** any metric is created if you want all
/// instruments to feed into a custom provider (e.g. one that also has an OTLP reader).
///
/// Returns `false` if the state was already initialized.
pub fn init_with_provider(
    exporter: opentelemetry_prometheus_text_exporter::PrometheusExporter,
    provider: SdkMeterProvider,
) -> bool {
    GLOBAL_STATE.set(GlobalMetricState { exporter, provider }).is_ok()
}

fn meter() -> opentelemetry::metrics::Meter {
    global_state().provider.meter("hopr")
}

/// Gathers all metrics in Prometheus text exposition format.
pub fn gather_all_metrics() -> MetricResult<String> {
    let state = global_state();
    let mut buf = Vec::new();
    state
        .exporter
        .export(&mut buf)
        .map_err(|e| MetricError(e.to_string()))?;
    String::from_utf8(buf).map_err(|e| MetricError(e.to_string()))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn labels_to_attributes(keys: &[String], values: &[&str]) -> Vec<KeyValue> {
    debug_assert_eq!(
        keys.len(),
        values.len(),
        "label key count ({}) must match value count ({})",
        keys.len(),
        values.len()
    );
    keys.iter()
        .zip(values.iter())
        .map(|(k, v)| KeyValue::new(k.clone(), v.to_string()))
        .collect()
}

/// Stores an `f64` in an `AtomicU64` via bit reinterpretation.
struct AtomicF64(AtomicU64);

impl AtomicF64 {
    fn new(val: f64) -> Self {
        Self(AtomicU64::new(val.to_bits()))
    }

    fn load(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }

    fn store(&self, val: f64) {
        self.0.store(val.to_bits(), Ordering::Relaxed);
    }

    /// Atomically swaps the value and returns the previous one.
    fn swap(&self, new: f64) -> f64 {
        f64::from_bits(self.0.swap(new.to_bits(), Ordering::Relaxed))
    }

    fn fetch_add(&self, delta: f64) -> f64 {
        loop {
            let current = self.0.load(Ordering::Relaxed);
            let current_f64 = f64::from_bits(current);
            let new_f64 = current_f64 + delta;
            if self
                .0
                .compare_exchange_weak(current, new_f64.to_bits(), Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return new_f64;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SimpleCounter
// ---------------------------------------------------------------------------

/// Represents a simple monotonic unsigned integer counter.
pub struct SimpleCounter {
    name: String,
    ctr: Counter<u64>,
    shadow: AtomicU64,
}

impl SimpleCounter {
    /// Creates a new integer counter with given name and description.
    pub fn new(name: &str, description: &str) -> MetricResult<Self> {
        let ctr = meter()
            .u64_counter(name.to_string())
            .with_description(description.to_string())
            .build();
        Ok(Self {
            name: name.to_string(),
            ctr,
            shadow: AtomicU64::new(0),
        })
    }

    /// Retrieves the value of the counter.
    pub fn get(&self) -> u64 {
        self.shadow.load(Ordering::Relaxed)
    }

    /// Increments the counter by the given number.
    pub fn increment_by(&self, by: u64) {
        self.ctr.add(by, &[]);
        self.shadow.fetch_add(by, Ordering::Relaxed);
    }

    /// Increments the counter by 1.
    pub fn increment(&self) {
        self.increment_by(1);
    }

    /// Returns the name of the counter given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

// ---------------------------------------------------------------------------
// MultiCounter
// ---------------------------------------------------------------------------

/// Represents a vector of named monotonic unsigned integer counters.
pub struct MultiCounter {
    name: String,
    labels: Vec<String>,
    ctr: Counter<u64>,
}

impl MultiCounter {
    /// Creates a new vector of integer counters with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> MetricResult<Self> {
        if labels.is_empty() {
            return Err(MetricError("at least a single label must be specified".into()));
        }
        let ctr = meter()
            .u64_counter(name.to_string())
            .with_description(description.to_string())
            .build();
        Ok(Self {
            name: name.to_string(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            ctr,
        })
    }

    /// Increments counter with given labels by the given number.
    pub fn increment_by(&self, label_values: &[&str], by: u64) {
        let attrs = labels_to_attributes(&self.labels, label_values);
        self.ctr.add(by, &attrs);
    }

    /// Increments counter with given labels by 1.
    pub fn increment(&self, label_values: &[&str]) {
        self.increment_by(label_values, 1);
    }

    /// Returns the name of the counter vector given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

// ---------------------------------------------------------------------------
// SimpleGauge
// ---------------------------------------------------------------------------

/// Represents a simple gauge with floating point values.
pub struct SimpleGauge {
    name: String,
    gauge: UpDownCounter<f64>,
    shadow: AtomicF64,
}

impl SimpleGauge {
    /// Creates a new gauge with given name and description.
    pub fn new(name: &str, description: &str) -> MetricResult<Self> {
        let gauge = meter()
            .f64_up_down_counter(name.to_string())
            .with_description(description.to_string())
            .build();
        Ok(Self {
            name: name.to_string(),
            gauge,
            shadow: AtomicF64::new(0.0),
        })
    }

    /// Increments the gauge by the given value.
    pub fn increment(&self, by: f64) {
        self.gauge.add(by, &[]);
        self.shadow.fetch_add(by);
    }

    /// Decrements the gauge by the given value.
    pub fn decrement(&self, by: f64) {
        self.gauge.add(-by, &[]);
        self.shadow.fetch_add(-by);
    }

    /// Sets the gauge to the given value.
    pub fn set(&self, value: f64) {
        let previous = self.shadow.swap(value);
        self.gauge.add(value - previous, &[]);
    }

    /// Retrieves the value of the gauge.
    pub fn get(&self) -> f64 {
        self.shadow.load()
    }

    /// Returns the name of the gauge given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

// ---------------------------------------------------------------------------
// MultiGauge
// ---------------------------------------------------------------------------

/// Represents a vector of gauges with floating point values.
pub struct MultiGauge {
    name: String,
    labels: Vec<String>,
    gauge: UpDownCounter<f64>,
    shadow: std::sync::RwLock<std::collections::HashMap<Vec<String>, AtomicF64>>,
}

impl MultiGauge {
    /// Creates a new vector of gauges with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> MetricResult<Self> {
        if labels.is_empty() {
            return Err(MetricError("at least a single label must be specified".into()));
        }
        let gauge = meter()
            .f64_up_down_counter(name.to_string())
            .with_description(description.to_string())
            .build();
        Ok(Self {
            name: name.to_string(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            gauge,
            shadow: std::sync::RwLock::new(std::collections::HashMap::new()),
        })
    }

    fn shadow_entry(&self, label_values: &[&str]) -> Vec<String> {
        label_values.iter().map(|s| s.to_string()).collect()
    }

    fn ensure_shadow(&self, key: &[String]) {
        {
            let read = self.shadow.read().unwrap();
            if read.contains_key(key) {
                return;
            }
        }
        let mut write = self.shadow.write().unwrap();
        write.entry(key.to_vec()).or_insert_with(|| AtomicF64::new(0.0));
    }

    /// Increments gauge with given labels by the given number.
    pub fn increment(&self, label_values: &[&str], by: f64) {
        let attrs = labels_to_attributes(&self.labels, label_values);
        self.gauge.add(by, &attrs);
        let key = self.shadow_entry(label_values);
        self.ensure_shadow(&key);
        let read = self.shadow.read().unwrap();
        if let Some(v) = read.get(&key) {
            v.fetch_add(by);
        }
    }

    /// Decrements gauge with given labels by the given number.
    pub fn decrement(&self, label_values: &[&str], by: f64) {
        let attrs = labels_to_attributes(&self.labels, label_values);
        self.gauge.add(-by, &attrs);
        let key = self.shadow_entry(label_values);
        self.ensure_shadow(&key);
        let read = self.shadow.read().unwrap();
        if let Some(v) = read.get(&key) {
            v.fetch_add(-by);
        }
    }

    /// Sets gauge with given labels to the given value.
    pub fn set(&self, label_values: &[&str], value: f64) {
        let key = self.shadow_entry(label_values);
        self.ensure_shadow(&key);
        let attrs = labels_to_attributes(&self.labels, label_values);
        let read = self.shadow.read().unwrap();
        if let Some(v) = read.get(&key) {
            let previous = v.swap(value);
            self.gauge.add(value - previous, &attrs);
        }
    }

    /// Retrieves the value of the specified gauge.
    pub fn get(&self, label_values: &[&str]) -> Option<f64> {
        let key = self.shadow_entry(label_values);
        let read = self.shadow.read().unwrap();
        read.get(&key).map(|v| v.load())
    }

    /// Returns the name of the gauge vector given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

// ---------------------------------------------------------------------------
// Timer
// ---------------------------------------------------------------------------

/// Macro to start a timer measurement on a histogram.
#[macro_export]
macro_rules! histogram_start_measure {
    // SimpleHistogram case
    ($v:ident) => {
        $v.start_measure()
    };
    // MultiHistogram case
    ($v:ident, $l:expr) => {
        $v.start_measure($l)
    };
}

/// Represents a timer handle.
pub struct SimpleTimer {
    start: std::time::Instant,
    labels: Option<Vec<KeyValue>>,
}

// ---------------------------------------------------------------------------
// SimpleHistogram
// ---------------------------------------------------------------------------

/// Represents a histogram with floating point values.
pub struct SimpleHistogram {
    name: String,
    hh: Histogram<f64>,
}

impl SimpleHistogram {
    /// Creates a new histogram with the given name, description and buckets.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(name: &str, description: &str, buckets: Vec<f64>) -> MetricResult<Self> {
        let hh = meter()
            .f64_histogram(name.to_string())
            .with_description(description.to_string())
            .with_boundaries(buckets)
            .build();
        Ok(Self {
            name: name.to_string(),
            hh,
        })
    }

    /// Records a value observation to the histogram.
    pub fn observe(&self, value: f64) {
        self.hh.record(value, &[]);
    }

    /// Starts a timer.
    pub fn start_measure(&self) -> SimpleTimer {
        SimpleTimer {
            start: std::time::Instant::now(),
            labels: None,
        }
    }

    /// Stops the given timer and records the elapsed duration in seconds to the histogram.
    pub fn record_measure(&self, timer: SimpleTimer) {
        self.hh.record(timer.start.elapsed().as_secs_f64(), &[]);
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        timer.start.elapsed().as_secs_f64()
    }

    /// Returns the name of the histogram given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

// ---------------------------------------------------------------------------
// MultiHistogram
// ---------------------------------------------------------------------------

/// Represents a vector of histograms with floating point values.
pub struct MultiHistogram {
    name: String,
    labels: Vec<String>,
    hh: Histogram<f64>,
}

impl MultiHistogram {
    /// Creates a new histogram with the given name, description, buckets and labels.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(name: &str, description: &str, buckets: Vec<f64>, labels: &[&str]) -> MetricResult<Self> {
        if labels.is_empty() {
            return Err(MetricError("at least a single label must be specified".into()));
        }
        let hh = meter()
            .f64_histogram(name.to_string())
            .with_description(description.to_string())
            .with_boundaries(buckets)
            .build();
        Ok(Self {
            name: name.to_string(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            hh,
        })
    }

    /// Starts a timer for a histogram with the given labels.
    pub fn start_measure(&self, label_values: &[&str]) -> MetricResult<SimpleTimer> {
        Ok(SimpleTimer {
            start: std::time::Instant::now(),
            labels: Some(labels_to_attributes(&self.labels, label_values)),
        })
    }

    /// Records a value observation to the histogram with the given labels.
    pub fn observe(&self, label_values: &[&str], value: f64) {
        let attrs = labels_to_attributes(&self.labels, label_values);
        self.hh.record(value, &attrs);
    }

    /// Stops the given timer and records the elapsed duration in seconds to the multi-histogram.
    pub fn record_measure(&self, timer: SimpleTimer) {
        let elapsed = timer.start.elapsed().as_secs_f64();
        let attrs = timer.labels.as_deref().unwrap_or(&[]);
        self.hh.record(elapsed, attrs);
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        timer.start.elapsed().as_secs_f64()
    }

    /// Returns the name of the histogram given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn simple_counter() -> anyhow::Result<()> {
        let counter = SimpleCounter::new("otel_my_ctr", "test counter")?;

        assert_eq!("otel_my_ctr", counter.name());

        counter.increment();
        assert_eq!(1, counter.get());

        counter.increment_by(9);
        assert_eq!(10, counter.get());

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_ctr"),
            "Prometheus text must contain counter name"
        );

        Ok(())
    }

    #[test]
    fn multi_counter() -> anyhow::Result<()> {
        let counter = MultiCounter::new("otel_my_mctr", "test multicounter", &["version"])?;

        assert_eq!("otel_my_mctr", counter.name());
        assert!(counter.labels().contains(&"version"));

        counter.increment_by(&["1.90.1"], 10);
        counter.increment_by(&["1.89.20"], 1);
        counter.increment_by(&["1.90.1"], 15);

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_mctr"),
            "Prometheus text must contain multi counter name"
        );
        assert!(
            metrics.contains("version=\"1.90.1\""),
            "Prometheus text must contain label value"
        );

        Ok(())
    }

    #[test]
    fn simple_gauge() -> anyhow::Result<()> {
        let gauge = SimpleGauge::new("otel_my_gauge", "test gauge")?;

        assert_eq!("otel_my_gauge", gauge.name());

        gauge.increment(10.0);
        assert_eq!(10.0, gauge.get());

        gauge.decrement(5.1);
        assert!((gauge.get() - 4.9).abs() < f64::EPSILON);

        gauge.set(100.0);
        assert_eq!(100.0, gauge.get());

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_gauge"),
            "Prometheus text must contain gauge name"
        );

        Ok(())
    }

    #[test]
    fn multi_gauge() -> anyhow::Result<()> {
        let gauge = MultiGauge::new("otel_my_mgauge", "test multigauge", &["version"])?;

        assert_eq!("otel_my_mgauge", gauge.name());
        assert!(gauge.labels().contains(&"version"));

        gauge.increment(&["1.90.1"], 10.0);
        gauge.increment(&["1.89.20"], 5.0);
        gauge.increment(&["1.90.1"], 15.0);
        gauge.decrement(&["1.89.20"], 2.0);

        assert_eq!(25.0, gauge.get(&["1.90.1"]).context("should be present")?);
        assert_eq!(3.0, gauge.get(&["1.89.20"]).context("should be present")?);

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_mgauge"),
            "Prometheus text must contain multi gauge name"
        );

        Ok(())
    }

    #[test]
    fn simple_histogram() -> anyhow::Result<()> {
        let histogram = SimpleHistogram::new("otel_my_histogram", "test histogram", vec![1.0, 2.0, 3.0, 4.0, 5.0])?;

        assert_eq!("otel_my_histogram", histogram.name());

        histogram.observe(2.0);
        histogram.observe(2.0);
        histogram.observe(1.0);
        histogram.observe(5.0);

        let timer = histogram_start_measure!(histogram);
        histogram.cancel_measure(timer);

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_histogram"),
            "Prometheus text must contain histogram name"
        );

        Ok(())
    }

    #[test]
    fn multi_histogram() -> anyhow::Result<()> {
        let histogram = MultiHistogram::new(
            "otel_my_mhistogram",
            "test histogram",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            &["version"],
        )?;

        assert_eq!("otel_my_mhistogram", histogram.name());
        assert!(histogram.labels().contains(&"version"));

        histogram.observe(&["1.90.0"], 2.0);
        histogram.observe(&["1.90.0"], 2.0);
        histogram.observe(&["1.90.0"], 1.0);
        histogram.observe(&["1.90.0"], 5.0);
        histogram.observe(&["1.89.20"], 10.0);

        let timer = histogram_start_measure!(histogram, &["1.90.0"])?;
        histogram.cancel_measure(timer);

        let metrics = gather_all_metrics().context("gather_all_metrics")?;
        assert!(
            metrics.contains("otel_my_mhistogram"),
            "Prometheus text must contain multi histogram name"
        );

        Ok(())
    }
}
