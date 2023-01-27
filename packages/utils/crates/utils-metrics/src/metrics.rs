use prometheus::core::Collector;
use prometheus::{
    Error, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramTimer, HistogramVec, IntCounter,
    IntCounterVec, Opts, TextEncoder,
};

use utils_misc::ok_or_str;

fn register_metric<M, C>(name: &str, desc: &str, creator: C) -> Result<M, String>
where
    M: Clone + Collector + 'static,
    C: Fn(Opts) -> prometheus::Result<M>,
{
    let metric = ok_or_str!(creator(Opts::new(name, desc)))?;

    ok_or_str!(prometheus::register(Box::new(metric.clone())))?;

    Ok(metric)
}

fn register_metric_vec<M, C>(
    name: &str,
    desc: &str,
    labels: &[&str],
    creator: C,
) -> Result<M, String>
where
    M: Clone + Collector + 'static,
    C: Fn(Opts, &[&str]) -> prometheus::Result<M>,
{
    if labels.len() == 0 {
        return Err("at least a single label must be specified".into());
    }

    let metric = ok_or_str!(creator(Opts::new(name, desc), labels))?;

    ok_or_str!(prometheus::register(Box::new(metric.clone())))?;

    Ok(metric)
}

/// Represents a simple monotonic unsigned integer counter.
/// Wrapper for IntCounter type
pub struct SimpleCounter {
    name: String,
    ctr: IntCounter,
}

impl SimpleCounter {
    /// Creates a new integer counter with given name and description
    pub fn new(name: &str, description: &str) -> Result<Self, String> {
        register_metric(name, description, IntCounter::with_opts).map(|m| Self {
            name: name.to_string(),
            ctr: m,
        })
    }

    /// Retrieves the value of the counter
    pub fn get(&self) -> u64 {
        self.ctr.get()
    }

    /// Increments the counter by the given number.
    pub fn increment(&self, by: u64) {
        self.ctr.inc_by(by)
    }

    /// Returns the name of the counter given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Represents a vector of named monotonic unsigned integer counters.
/// Wrapper for IntCounterVec type
pub struct MultiCounter {
    name: String,
    labels: Vec<String>,
    ctr: IntCounterVec,
}

impl MultiCounter {
    /// Creates a new vector of integer counters with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> Result<Self, String> {
        register_metric_vec(name, description, labels, IntCounterVec::new).map(|m| Self {
            name: name.to_string(),
            labels: Vec::from(labels).iter().map(|s| String::from(*s)).collect(),
            ctr: m,
        })
    }

    /// Increments counter with given labels by the given number.
    pub fn increment(&self, label_values: &[&str], by: u64) {
        if let Ok(c) = self.ctr.get_metric_with_label_values(label_values) {
            c.inc_by(by)
        }
    }

    /// Retrieves the value of the specified counter
    pub fn get(&self, label_values: &[&str]) -> Option<u64> {
        self.ctr
            .get_metric_with_label_values(label_values)
            .map(|c| c.get())
            .ok()
    }

    /// Returns the name of the counter vector given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

/// Represents a simple gauge with floating point values.
/// Wrapper for Gauge type
pub struct SimpleGauge {
    name: String,
    gg: Gauge,
}

impl SimpleGauge {
    /// Creates a new gauge with given name and description.
    pub fn new(name: &str, description: &str) -> Result<Self, String> {
        register_metric(name, description, Gauge::with_opts).map(|m| Self {
            name: name.to_string(),
            gg: m,
        })
    }

    /// Increments the gauge by the given value.
    pub fn increment(&self, by: f64) {
        self.gg.add(by)
    }

    /// Decrements the gauge by the given value.
    pub fn decrement(&self, by: f64) {
        self.gg.sub(by)
    }

    /// Sets the gauge to the given value.
    pub fn set(&self, value: f64) {
        self.gg.set(value)
    }

    /// Retrieves the value of the gauge
    pub fn get(&self) -> f64 {
        self.gg.get()
    }

    /// Returns the name of the gauge given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Represents a vector of gauges with floating point values.
/// Wrapper for GaugeVec type
pub struct MultiGauge {
    name: String,
    labels: Vec<String>,
    ctr: GaugeVec,
}

impl MultiGauge {
    /// Creates a new vector of gauges with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> Result<Self, String> {
        register_metric_vec(name, description, labels, GaugeVec::new).map(|m| Self {
            name: name.to_string(),
            labels: Vec::from(labels).iter().map(|s| String::from(*s)).collect(),
            ctr: m,
        })
    }

    /// Increments gauge with given labels by the given number.
    pub fn increment(&self, label_values: &[&str], by: f64) {
        if let Ok(c) = self.ctr.get_metric_with_label_values(label_values) {
            c.add(by)
        }
    }

    /// Decrements gauge with given labels by the given number.
    pub fn decrement(&self, label_values: &[&str], by: f64) {
        if let Ok(c) = self.ctr.get_metric_with_label_values(label_values) {
            c.sub(by)
        }
    }

    /// Sets gauge with given labels to the given value.
    pub fn set(&self, label_values: &[&str], value: f64) {
        if let Ok(c) = self.ctr.get_metric_with_label_values(label_values) {
            c.set(value)
        }
    }

    /// Retrieves the value of the specified counter
    pub fn get(&self, label_values: &[&str]) -> Option<f64> {
        self.ctr
            .get_metric_with_label_values(label_values)
            .map(|c| c.get())
            .ok()
    }

    /// Returns the name of the gauge vector given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

/// Represents a histogram with floating point values.
/// Wrapper for Histogram type
pub struct SimpleHistogram {
    name: String,
    hh: Histogram,
}

/// Represents a timer handle.
pub struct SimpleTimer {
    histogram_timer: HistogramTimer,
}

impl SimpleHistogram {
    /// Creates a new histogram with the given name, description and buckets.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(name: &str, description: &str, buckets: Vec<f64>) -> Result<Self, String> {
        let mut opts = HistogramOpts::new(name, description);
        if !buckets.is_empty() {
            opts = opts.buckets(buckets);
        }

        let metric = ok_or_str!(Histogram::with_opts(opts))?;

        ok_or_str!(prometheus::register(Box::new(metric.clone())))?;

        Ok(Self {
            name: name.to_string(),
            hh: metric,
        })
    }

    /// Records a value observation to the histogram.
    pub fn observe(&self, value: f64) {
        self.hh.observe(value)
    }

    /// Starts a timer.
    #[allow(dead_code)]
    pub fn start_measure(&self) -> SimpleTimer {
        SimpleTimer {
            histogram_timer: self.hh.start_timer(),
        }
    }

    /// Stops the given timer and records the elapsed duration in seconds to the histogram.
    #[allow(dead_code)]
    pub fn record_measure(&self, timer: SimpleTimer) {
        timer.histogram_timer.observe_duration()
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    #[allow(dead_code)]
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        timer.histogram_timer.stop_and_discard()
    }

    /// Get all samples count
    pub fn get_sample_count(&self) -> u64 {
        self.hh.get_sample_count()
    }

    /// Get all samples sum
    pub fn get_sample_sum(&self) -> f64 {
        self.hh.get_sample_sum()
    }

    /// Returns the name of the histogram given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Represents a vector of histograms with floating point values.
/// Wrapper for HistogramVec type
pub struct MultiHistogram {
    name: String,
    labels: Vec<String>,
    hh: HistogramVec,
}

impl MultiHistogram {
    /// Creates a new histogram with the given name, description and buckets.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(
        name: &str,
        description: &str,
        buckets: Vec<f64>,
        labels: &[&str],
    ) -> Result<Self, String> {
        let mut opts = HistogramOpts::new(name, description);
        if !buckets.is_empty() {
            opts = opts.buckets(buckets);
        }

        let metric = ok_or_str!(HistogramVec::new(opts, labels))?;

        ok_or_str!(prometheus::register(Box::new(metric.clone())))?;

        Ok(Self {
            name: name.to_string(),
            labels: Vec::from(labels).iter().map(|s| String::from(*s)).collect(),
            hh: metric,
        })
    }

    /// Records a value observation to the histogram with the given labels.
    pub fn observe(&self, label_values: &[&str], value: f64) {
        if let Ok(c) = self.hh.get_metric_with_label_values(label_values) {
            c.observe(value)
        }
    }

    /// Starts a timer for a histogram with the given labels.
    #[allow(dead_code)]
    pub fn start_measure(&self, label_values: &[&str]) -> Result<SimpleTimer, Error> {
        self.hh
            .get_metric_with_label_values(label_values)
            .map(|h| SimpleTimer {
                histogram_timer: h.start_timer(),
            })
    }

    /// Stops the given timer and records the elapsed duration in seconds to the histogram.
    #[allow(dead_code)]
    pub fn record_measure(&self, timer: SimpleTimer) {
        timer.histogram_timer.observe_duration()
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    #[allow(dead_code)]
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        timer.histogram_timer.stop_and_discard()
    }

    /// Get all samples count with given labels
    pub fn get_sample_count(&self, label_values: &[&str]) -> Option<u64> {
        self.hh
            .get_metric_with_label_values(label_values)
            .map(|c| c.get_sample_count())
            .ok()
    }

    /// Get all samples sum with given labels
    pub fn get_sample_sum(&self, label_values: &[&str]) -> Option<f64> {
        self.hh
            .get_metric_with_label_values(label_values)
            .map(|c| c.get_sample_sum())
            .ok()
    }

    /// Returns the name of the histogram given at construction.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the labels of the histogram given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

/// Gathers all the global Prometheus metrics.
pub fn gather_all_metrics() -> Result<String, String> {
    // Simply gather all global metric families
    let metric_families = prometheus::gather();

    // ... and encode them
    let encoder = TextEncoder::new();
    ok_or_str!(encoder.encode_to_string(&metric_families))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = SimpleCounter::new("my_ctr", "test counter").unwrap();

        assert_eq!("my_ctr", counter.name());

        counter.increment(1);

        assert_eq!(1, counter.get());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_ctr 1"));
    }

    #[test]
    fn test_multi_counter() {
        let counter = MultiCounter::new("my_mctr", "test multicounter", &["version"]).unwrap();

        assert_eq!("my_mctr", counter.name());
        assert!(counter.labels().contains(&"version"));

        counter.increment(&["1.90.1"], 10);
        counter.increment(&["1.89.20"], 1);
        counter.increment(&["1.90.1"], 15);

        assert_eq!(25, counter.get(&["1.90.1"]).unwrap());
        assert_eq!(1, counter.get(&["1.89.20"]).unwrap());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_mctr{version=\"1.90.1\"} 25"));
        assert!(metrics.contains("my_mctr{version=\"1.89.20\"} 1"));
    }

    #[test]
    fn test_gauge() {
        let gauge = SimpleGauge::new("my_gauge", "test gauge").unwrap();

        assert_eq!("my_gauge", gauge.name());

        gauge.increment(10.0);

        assert_eq!(10.0, gauge.get());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_gauge 10"));

        gauge.decrement(5.1);

        assert_eq!(4.9, gauge.get());

        let metrics2 = gather_all_metrics().unwrap();
        assert!(metrics2.contains("my_gauge 4.9"));
    }

    #[test]
    fn test_multi_gauge() {
        let gauge = MultiGauge::new("my_mgauge", "test multicounter", &["version"]).unwrap();

        assert_eq!("my_mgauge", gauge.name());
        assert!(gauge.labels().contains(&"version"));

        gauge.increment(&["1.90.1"], 10.0);
        gauge.increment(&["1.89.20"], 5.0);
        gauge.increment(&["1.90.1"], 15.0);
        gauge.decrement(&["1.89.20"], 2.0);

        assert_eq!(25.0, gauge.get(&["1.90.1"]).unwrap());
        assert_eq!(3.0, gauge.get(&["1.89.20"]).unwrap());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_mgauge{version=\"1.90.1\"} 25"));
        assert!(metrics.contains("my_mgauge{version=\"1.89.20\"} 3"));
    }

    #[test]
    fn test_histogram() {
        let histogram = SimpleHistogram::new(
            "my_histogram",
            "test histogram",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        )
        .unwrap();

        assert_eq!("my_histogram", histogram.name());

        histogram.observe(2.0);
        histogram.observe(2.0);
        histogram.observe(1.0);
        histogram.observe(5.0);

        assert_eq!(4, histogram.get_sample_count());
        assert_eq!(10.0, histogram.get_sample_sum());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_histogram_bucket{le=\"1\"} 1"));
        assert!(metrics.contains("my_histogram_bucket{le=\"2\"} 3"));
        assert!(metrics.contains("my_histogram_bucket{le=\"3\"} 3"));
        assert!(metrics.contains("my_histogram_bucket{le=\"4\"} 3"));
        assert!(metrics.contains("my_histogram_bucket{le=\"5\"} 4"));
    }

    #[test]
    fn test_multi_histogram() {
        let histogram = MultiHistogram::new(
            "my_mhistogram",
            "test histogram",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            &["version"],
        )
        .unwrap();

        assert_eq!("my_mhistogram", histogram.name());
        assert!(histogram.labels().contains(&"version"));

        histogram.observe(&["1.90.0"], 2.0);
        histogram.observe(&["1.90.0"], 2.0);
        histogram.observe(&["1.90.0"], 1.0);
        histogram.observe(&["1.90.0"], 5.0);
        histogram.observe(&["1.89.20"], 10.0);

        assert_eq!(1, histogram.get_sample_count(&["1.89.20"]).unwrap());
        assert_eq!(10.0, histogram.get_sample_sum(&["1.89.20"]).unwrap());

        assert_eq!(4, histogram.get_sample_count(&["1.90.0"]).unwrap());
        assert_eq!(10.0, histogram.get_sample_sum(&["1.90.0"]).unwrap());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.90.0\",le=\"1\"} 1"));
        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.90.0\",le=\"2\"} 3"));
        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.90.0\",le=\"3\"} 3"));
        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.90.0\",le=\"4\"} 3"));
        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.90.0\",le=\"5\"} 4"));

        assert!(metrics.contains("my_mhistogram_bucket{version=\"1.89.20\",le=\"+Inf\"} 1"));
    }
}

/// Bindings for JS/TS
#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::{Date, JsString};
    use utils_misc::{convert_from_jstrvec, convert_to_jstrvec, ok_or_jserr};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    //// SimpleCounter

    #[wasm_bindgen]
    pub struct SimpleCounter {
        w: super::SimpleCounter,
    }

    #[wasm_bindgen]
    pub fn create_counter(name: &str, description: &str) -> Result<SimpleCounter, JsValue> {
        ok_or_jserr!(super::SimpleCounter::new(name, description).map(|c| SimpleCounter { w: c }))
    }

    #[wasm_bindgen]
    impl SimpleCounter {
        pub fn increment_by(&self, by: u64) {
            self.w.increment(by);
        }

        pub fn increment(&self) {
            self.w.increment(1)
        }

        pub fn get(&self) -> u64 {
            self.w.get()
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    //// MultiCounter

    #[wasm_bindgen]
    pub struct MultiCounter {
        w: super::MultiCounter,
    }

    #[wasm_bindgen]
    pub fn create_multi_counter(
        name: &str,
        description: &str,
        labels: Vec<JsString>,
    ) -> Result<MultiCounter, JsValue> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(super::MultiCounter::new(name, description, bind.as_slice())
            .map(|c| MultiCounter { w: c }))
    }

    #[wasm_bindgen]
    impl MultiCounter {
        pub fn increment_by(&self, label_values: Vec<JsString>, by: u64) {
            convert_from_jstrvec!(label_values, bind);
            self.w.increment(bind.as_slice(), by);
        }

        pub fn increment(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.w.increment(bind.as_slice(), 1)
        }

        pub fn get(&self, label_values: Vec<JsString>) -> Result<u64, JsValue> {
            convert_from_jstrvec!(label_values, bind);
            self.w
                .get(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn labels(&self) -> Vec<JsString> {
            convert_to_jstrvec!(self.w.labels)
        }
    }

    //// SimpleGauge

    #[wasm_bindgen]
    pub struct SimpleGauge {
        w: super::SimpleGauge,
    }

    #[wasm_bindgen]
    pub fn create_gauge(name: &str, description: &str) -> Result<SimpleGauge, JsValue> {
        ok_or_jserr!(super::SimpleGauge::new(name, description).map(|c| SimpleGauge { w: c }))
    }

    #[wasm_bindgen]
    impl SimpleGauge {
        pub fn increment_by(&self, by: f64) {
            self.w.increment(by);
        }

        pub fn increment(&self) {
            self.w.increment(1.0);
        }

        pub fn decrement_by(&self, by: f64) {
            self.w.decrement(by);
        }

        pub fn decrement(&self) {
            self.w.decrement(1.0);
        }

        pub fn set(&self, value: f64) {
            self.w.set(value)
        }

        pub fn get(&self) -> f64 {
            self.w.get()
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    //// MultiGauge

    #[wasm_bindgen]
    pub struct MultiGauge {
        w: super::MultiGauge,
    }

    #[wasm_bindgen]
    pub fn create_multi_gauge(
        name: &str,
        description: &str,
        labels: Vec<JsString>,
    ) -> Result<MultiGauge, JsValue> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(
            super::MultiGauge::new(name, description, bind.as_slice()).map(|c| MultiGauge { w: c })
        )
    }

    #[wasm_bindgen]
    impl MultiGauge {
        pub fn increment_by(&self, label_values: Vec<JsString>, by: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.w.increment(bind.as_slice(), by);
        }

        pub fn increment(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.w.increment(bind.as_slice(), 1.0)
        }

        pub fn decrement_by(&self, label_values: Vec<JsString>, by: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.w.decrement(bind.as_slice(), by);
        }

        pub fn decrement(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.w.decrement(bind.as_slice(), 1.0)
        }

        pub fn set(&self, label_values: Vec<JsString>, value: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.w.set(bind.as_slice(), value);
        }

        pub fn get(&self, label_values: Vec<JsString>) -> Result<f64, JsValue> {
            convert_from_jstrvec!(label_values, bind);
            self.w
                .get(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn labels(&self) -> Vec<JsString> {
            convert_to_jstrvec!(self.w.labels)
        }
    }

    //// SimpleHistogram

    #[wasm_bindgen]
    pub struct SimpleHistogram {
        w: super::SimpleHistogram,
    }

    #[wasm_bindgen]
    pub fn create_histogram(name: &str, description: &str) -> Result<SimpleHistogram, JsValue> {
        create_histogram_with_buckets(name, description, &[] as &[f64; 0])
    }

    #[wasm_bindgen]
    pub fn create_histogram_with_buckets(
        name: &str,
        description: &str,
        buckets: &[f64],
    ) -> Result<SimpleHistogram, JsValue> {
        ok_or_jserr!(
            super::SimpleHistogram::new(name, description, buckets.into())
                .map(|c| SimpleHistogram { w: c })
        )
    }

    /// Currently the SimpleTimer is NOT a wrapper for HistogramTimer,
    /// but rather implements the timer logic using js_sys::Date to achieve a similar functionality.
    /// This is because WASM does not support system time functionality from the Rust stdlib.
    #[wasm_bindgen]
    pub struct SimpleTimer {
        start: f64,
        labels: Vec<String>,
    }

    impl SimpleTimer {
        fn new(label_values: Vec<JsString>) -> Self {
            SimpleTimer {
                start: Self::now(),
                labels: label_values.iter().map(String::from).collect(),
            }
        }

        /// Current Unix timestamp (in seconds) using js_sys::Date
        fn now() -> f64 {
            Date::now() / 1000.0
        }

        /// Computes the time elapsed since the creation of this timer.
        fn diff(&self) -> f64 {
            Self::now() - self.start
        }
    }

    #[wasm_bindgen]
    impl SimpleHistogram {
        pub fn observe(&self, value: f64) {
            self.w.observe(value)
        }

        pub fn start_measure(&self) -> SimpleTimer {
            SimpleTimer::new(vec![])
        }

        pub fn record_measure(&self, timer: SimpleTimer) {
            self.w.observe(timer.diff())
        }

        pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
            timer.diff()
        }

        pub fn get_sample_count(&self) -> u64 {
            self.w.get_sample_count()
        }

        pub fn get_sample_sum(&self) -> f64 {
            self.w.get_sample_sum()
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    //// MultiHistogram

    #[wasm_bindgen]
    pub struct MultiHistogram {
        w: super::MultiHistogram,
    }

    #[wasm_bindgen]
    pub fn create_multi_histogram(
        name: &str,
        description: &str,
        labels: Vec<JsString>,
    ) -> Result<MultiHistogram, JsValue> {
        create_multi_histogram_with_buckets(name, description, &[] as &[f64; 0], labels)
    }

    #[wasm_bindgen]
    pub fn create_multi_histogram_with_buckets(
        name: &str,
        description: &str,
        buckets: &[f64],
        labels: Vec<JsString>,
    ) -> Result<MultiHistogram, JsValue> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(
            super::MultiHistogram::new(name, description, buckets.into(), bind.as_slice())
                .map(|c| MultiHistogram { w: c })
        )
    }

    #[wasm_bindgen]
    impl MultiHistogram {
        pub fn observe(&self, label_values: Vec<JsString>, value: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.w.observe(bind.as_slice(), value)
        }

        pub fn start_measure(&self, label_values: Vec<JsString>) -> SimpleTimer {
            SimpleTimer::new(label_values)
        }

        pub fn record_measure(&self, timer: SimpleTimer) {
            convert_from_jstrvec!(timer.labels, bind);
            self.w.observe(bind.as_slice(), timer.diff())
        }

        pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
            timer.diff()
        }

        pub fn get_sample_count(&self, label_values: Vec<JsString>) -> Result<u64, JsValue> {
            convert_from_jstrvec!(label_values, bind);
            self.w
                .get_sample_count(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }

        pub fn get_sample_sum(&self, label_values: Vec<JsString>) -> Result<f64, JsValue> {
            convert_from_jstrvec!(label_values, bind);
            self.w
                .get_sample_sum(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }

        pub fn labels(&self) -> Vec<JsString> {
            convert_to_jstrvec!(self.w.labels)
        }
    }

    #[wasm_bindgen]
    pub fn gather_all_metrics() -> Result<String, JsValue> {
        ok_or_jserr!(super::gather_all_metrics())
    }
}
