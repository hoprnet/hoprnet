use prometheus::core::Collector;
use prometheus::{
    Gauge, GaugeVec, Histogram, HistogramOpts, HistogramTimer, HistogramVec, IntCounter, IntCounterVec, Opts,
    TextEncoder,
};
use regex::Regex;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

/// Gathers all the global Prometheus metrics.
pub fn gather_all_metrics() -> prometheus::Result<String> {
    let families = prometheus::gather();

    let encoder = TextEncoder::new();
    encoder.encode_to_string(&families)
}

/// A naive merging method for two serialized metric registries.
/// It performs union of the sets, removing those metrics which have the same name and type.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn merge_encoded_metrics(metrics1: &str, metrics2: &str) -> String {
    let mut merged_metrics = BTreeMap::new();
    let metric_expr =
        Regex::new(r"(?:# HELP)\s(?P<name>\w+)\s.+\s+(?:# TYPE)\s\w+\s(?P<type>\w+)\s+(?:[^#]+\s)+").unwrap();

    let merged_texts = metrics1.to_owned() + metrics2;

    // Search for all metrics in the merged texts and skip those with duplicate name and type, first comes first served.
    for complete_metric in metric_expr.captures_iter(&merged_texts) {
        let metric_key = format!("{}~{}", &complete_metric["name"], &complete_metric["type"]);
        if let Entry::Vacant(metric) = merged_metrics.entry(metric_key) {
            metric.insert(complete_metric[0].to_string());
        }
    }

    // Output metrics sorted lexicographically by name
    merged_metrics.values().fold("".into(), |mut a, b| {
        a.reserve(b.len()); // pre-alloc space on LHS for better efficiency
        a.push_str(b);
        a
    })
}

fn register_metric<M, C>(name: &str, desc: &str, creator: C) -> prometheus::Result<M>
where
    M: Clone + Collector + 'static,
    C: Fn(Opts) -> prometheus::Result<M>,
{
    let metric = creator(Opts::new(name, desc))?;

    prometheus::register(Box::new(metric.clone()))?;

    Ok(metric)
}

fn register_metric_vec<M, C>(name: &str, desc: &str, labels: &[&str], creator: C) -> prometheus::Result<M>
where
    M: Clone + Collector + 'static,
    C: Fn(Opts, &[&str]) -> prometheus::Result<M>,
{
    if labels.is_empty() {
        return Err(prometheus::Error::Msg(
            "at least a single label must be specified".into(),
        ));
    }

    let metric = creator(Opts::new(name, desc), labels)?;

    prometheus::register(Box::new(metric.clone()))?;

    Ok(metric)
}

/// Represents a simple monotonic unsigned integer counter.
/// Wrapper for IntCounter type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SimpleCounter {
    name: String,
    ctr: IntCounter,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl SimpleCounter {
    /// Retrieves the value of the counter
    pub fn get(&self) -> u64 {
        self.ctr.get()
    }

    /// Increments the counter by the given number.
    pub fn increment_by(&self, by: u64) {
        self.ctr.inc_by(by)
    }

    /// Increments the counter by 1
    pub fn increment(&self) {
        self.increment_by(1)
    }

    /// Returns the name of the counter given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl SimpleCounter {
    /// Creates a new integer counter with given name and description
    pub fn new(name: &str, description: &str) -> prometheus::Result<Self> {
        register_metric(name, description, IntCounter::with_opts).map(|m| Self {
            name: name.to_string(),
            ctr: m,
        })
    }
}

/// Represents a vector of named monotonic unsigned integer counters.
/// Wrapper for IntCounterVec type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MultiCounter {
    name: String,
    labels: Vec<String>,
    ctr: IntCounterVec,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MultiCounter {
    /// Returns the name of the counter vector given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl MultiCounter {
    /// Creates a new vector of integer counters with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> prometheus::Result<Self> {
        register_metric_vec(name, description, labels, IntCounterVec::new).map(|m| Self {
            name: name.to_string(),
            labels: Vec::from(labels).iter().map(|s| String::from(*s)).collect(),
            ctr: m,
        })
    }

    /// Increments counter with given labels by the given number.
    pub fn increment_by(&self, label_values: &[&str], by: u64) {
        if let Ok(c) = self.ctr.get_metric_with_label_values(label_values) {
            c.inc_by(by)
        }
    }

    /// Increments counter with given labels by 1.
    pub fn increment(&self, label_values: &[&str]) {
        self.increment_by(label_values, 1)
    }

    /// Retrieves the value of the specified counter
    pub fn get(&self, label_values: &[&str]) -> Option<u64> {
        self.ctr
            .get_metric_with_label_values(label_values)
            .map(|c| c.get())
            .ok()
    }

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

/// Represents a simple gauge with floating point values.
/// Wrapper for Gauge type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug)]
pub struct SimpleGauge {
    name: String,
    gg: Gauge,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl SimpleGauge {
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
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl SimpleGauge {
    /// Creates a new gauge with given name and description.
    pub fn new(name: &str, description: &str) -> prometheus::Result<Self> {
        register_metric(name, description, Gauge::with_opts).map(|m| Self {
            name: name.to_string(),
            gg: m,
        })
    }
}

/// Represents a vector of gauges with floating point values.
/// Wrapper for GaugeVec type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug)]
pub struct MultiGauge {
    name: String,
    labels: Vec<String>,
    ctr: GaugeVec,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MultiGauge {
    /// Returns the name of the gauge vector given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl MultiGauge {
    /// Creates a new vector of gauges with given name, description and counter labels.
    pub fn new(name: &str, description: &str, labels: &[&str]) -> prometheus::Result<Self> {
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

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

/// Convenience helper macro for creating time measurements to a histogram.
/// The macro will distinguish between WASM and non-WASM case automatically and will evaluate
/// the SimpleTimer object.
/// First argument is either SimpleHistogram or MultiHistogram.
/// If MultiHistogram has been supplied, an additional argument with labels must be passed.
#[cfg(all(feature = "wasm", not(test)))]
#[macro_export]
macro_rules! histogram_start_measure {
    // SimpleHistogram case
    ($v:ident) => {
        $v.wasm_start_measure()
    };
    // MultiHistogram case
    ($v:ident, $l:expr) => {
        $v.wasm_start_measure($l.iter().map(|s| js_sys::JsString::from(*s)).collect())
            .map_err(|_| prometheus::Error::Msg("invalid label".into()))
    };
}

#[cfg(any(not(feature = "wasm"), test))]
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

enum TimerVariant {
    Native(HistogramTimer),
    #[cfg(feature = "wasm")]
    Wasm {
        start_ts: f64,
        new_ts: fn() -> f64,
        labels: Vec<String>,
    },
}

/// Represents a timer handle.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SimpleTimer {
    inner: TimerVariant,
}

/// Represents a histogram with floating point values.
/// Wrapper for Histogram type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SimpleHistogram {
    name: String,
    hh: Histogram,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl SimpleHistogram {
    /// Records a value observation to the histogram.
    pub fn observe(&self, value: f64) {
        self.hh.observe(value)
    }

    /// Stops the given timer and records the elapsed duration in seconds to the histogram.
    pub fn record_measure(&self, timer: SimpleTimer) {
        match timer.inner {
            TimerVariant::Native(timer) => timer.observe_duration(),
            #[cfg(feature = "wasm")]
            TimerVariant::Wasm { start_ts, new_ts, .. } => self.hh.observe(new_ts() - start_ts),
        }
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        match timer.inner {
            TimerVariant::Native(timer) => timer.stop_and_discard(),
            #[cfg(feature = "wasm")]
            TimerVariant::Wasm { start_ts, new_ts, .. } => new_ts() - start_ts,
        }
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
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl SimpleHistogram {
    /// Creates a new histogram with the given name, description and buckets.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(name: &str, description: &str, buckets: Vec<f64>) -> prometheus::Result<Self> {
        let mut opts = HistogramOpts::new(name, description);
        if !buckets.is_empty() {
            opts = opts.buckets(buckets);
        }

        let metric = Histogram::with_opts(opts)?;

        prometheus::register(Box::new(metric.clone()))?;

        Ok(Self {
            name: name.to_string(),
            hh: metric,
        })
    }

    /// Starts a timer.
    pub fn start_measure(&self) -> SimpleTimer {
        SimpleTimer {
            inner: TimerVariant::Native(self.hh.start_timer()),
        }
    }
}

/// Represents a vector of histograms with floating point values.
/// Wrapper for HistogramVec type
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MultiHistogram {
    name: String,
    labels: Vec<String>,
    hh: HistogramVec,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MultiHistogram {
    /// Stops the given timer and records the elapsed duration in seconds to the multi-histogram.
    pub fn record_measure(&self, timer: SimpleTimer) {
        match timer.inner {
            TimerVariant::Native(timer) => timer.observe_duration(),
            #[cfg(feature = "wasm")]
            TimerVariant::Wasm {
                start_ts,
                new_ts,
                labels,
            } => {
                if let Ok(h) = self
                    .hh
                    .get_metric_with_label_values(&labels.iter().map(String::as_str).collect::<Vec<&str>>())
                {
                    h.observe(new_ts() - start_ts)
                }
            }
        }
    }

    /// Stops the given timer and discards the measured duration in seconds and returns it.
    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        match timer.inner {
            TimerVariant::Native(timer) => timer.stop_and_discard(),
            #[cfg(feature = "wasm")]
            TimerVariant::Wasm { start_ts, new_ts, .. } => new_ts() - start_ts,
        }
    }

    /// Returns the name of the histogram given at construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl MultiHistogram {
    /// Creates a new histogram with the given name, description and buckets.
    /// If no buckets are specified, they will be defined automatically.
    /// The +Inf bucket is always added automatically.
    pub fn new(name: &str, description: &str, buckets: Vec<f64>, labels: &[&str]) -> prometheus::Result<Self> {
        let mut opts = HistogramOpts::new(name, description);
        if !buckets.is_empty() {
            opts = opts.buckets(buckets);
        }

        let metric = HistogramVec::new(opts, labels)?;

        prometheus::register(Box::new(metric.clone()))?;

        Ok(Self {
            name: name.to_string(),
            labels: Vec::from(labels).iter().map(|s| String::from(*s)).collect(),
            hh: metric,
        })
    }

    /// Starts a timer for a histogram with the given labels.
    pub fn start_measure(&self, label_values: &[&str]) -> prometheus::Result<SimpleTimer> {
        self.hh.get_metric_with_label_values(label_values).map(|h| SimpleTimer {
            inner: TimerVariant::Native(h.start_timer()),
        })
    }

    /// Records a value observation to the histogram with the given labels.
    pub fn observe(&self, label_values: &[&str], value: f64) {
        if let Ok(c) = self.hh.get_metric_with_label_values(label_values) {
            c.observe(value)
        }
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

    /// Returns the labels of the counters given at construction.
    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = SimpleCounter::new("my_ctr", "test counter").unwrap();

        assert_eq!("my_ctr", counter.name());

        counter.increment();

        assert_eq!(1, counter.get());

        let metrics = gather_all_metrics().unwrap();
        assert!(metrics.contains("my_ctr 1"));
    }

    #[test]
    fn test_multi_counter() {
        let counter = MultiCounter::new("my_mctr", "test multicounter", &["version"]).unwrap();

        assert_eq!("my_mctr", counter.name());
        assert!(counter.labels().contains(&"version"));

        counter.increment_by(&["1.90.1"], 10);
        counter.increment_by(&["1.89.20"], 1);
        counter.increment_by(&["1.90.1"], 15);

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
        let histogram = SimpleHistogram::new("my_histogram", "test histogram", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();

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

        let timer = histogram_start_measure!(histogram);
        histogram.cancel_measure(timer);
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

        let timer = histogram_start_measure!(histogram, &["1.90.0"]).unwrap();
        histogram.cancel_measure(timer);
    }

    #[test]
    fn test_merging() {
        let counter = SimpleCounter::new("b_my_test_ctr", "test counter").unwrap();
        counter.increment();

        let multi_gauge = MultiGauge::new("c_mgauge", "test mgauge", &["version", "method"]).unwrap();
        multi_gauge.increment(&["1.10.11", "get"], 3.0);
        multi_gauge.increment(&["1.10.11", "post"], 1.0);

        let metrics1 = gather_all_metrics().unwrap();

        let counter2 = SimpleCounter::new("a_my_test_ctr_2", "test counter 2").unwrap();
        counter2.increment_by(2);

        let histogram = SimpleHistogram::new("b_histogram", "test histogram", vec![0.5, 1.0, 5.0]).unwrap();
        histogram.observe(0.3);

        let metrics2 = gather_all_metrics().unwrap();

        let res1 = merge_encoded_metrics(&metrics1, &metrics2);
        assert!(res1.contains("b_my_test_ctr"));
        assert_eq!(3, res1.match_indices("b_my_test_ctr").count());

        assert!(res1.contains("a_my_test_ctr_2"));
        assert_eq!(3, res1.match_indices("a_my_test_ctr_2").count());

        // Metrics must be sorted lexicographically by their names
        assert!(res1.find("b_my_test_ctr").unwrap() > res1.find("a_my_test_ctr_2").unwrap());
        assert!(res1.find("b_my_test_ctr").unwrap() > res1.find("b_histogram").unwrap());
        assert!(res1.find("c_mgauge").unwrap() > res1.find("b_my_test_ctr").unwrap());

        // Test degenerate cases

        let res2 = merge_encoded_metrics(&metrics1, &metrics1);
        assert!(res2.contains("b_my_test_ctr"));
        assert_eq!(3, res2.match_indices("b_my_test_ctr").count());

        let res3 = merge_encoded_metrics(&metrics1, "");
        assert_eq!(metrics1, res3);

        let res4 = merge_encoded_metrics("", &metrics1);
        assert_eq!(metrics1, res4);

        assert!(merge_encoded_metrics("", "").trim().is_empty());
    }
}

/// Bindings for JS/TS
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::metrics::TimerVariant::Wasm;
    use crate::metrics::{
        MultiCounter, MultiGauge, MultiHistogram, SimpleCounter, SimpleGauge, SimpleHistogram, SimpleTimer,
    };
    use js_sys::JsString;
    use utils_misc::utils::wasm::JsResult;
    use utils_misc::{convert_from_jstrvec, ok_or_jserr};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn gather_all_metrics() -> JsResult<String> {
        ok_or_jserr!(super::gather_all_metrics())
    }

    #[wasm_bindgen]
    pub fn create_counter(name: &str, description: &str) -> JsResult<SimpleCounter> {
        ok_or_jserr!(SimpleCounter::new(name, description))
    }

    #[wasm_bindgen]
    pub fn create_multi_counter(name: &str, description: &str, labels: Vec<JsString>) -> JsResult<MultiCounter> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(MultiCounter::new(name, description, bind.as_slice()))
    }

    #[wasm_bindgen]
    impl MultiCounter {
        #[wasm_bindgen(js_name = "increment_by")]
        pub fn _increment_by(&self, label_values: Vec<JsString>, by: u64) {
            convert_from_jstrvec!(label_values, bind);
            self.increment_by(bind.as_slice(), by);
        }

        #[wasm_bindgen(js_name = "increment")]
        pub fn _increment(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.increment(bind.as_slice())
        }

        #[wasm_bindgen(js_name = "get")]
        pub fn _get(&self, label_values: Vec<JsString>) -> JsResult<u64> {
            convert_from_jstrvec!(label_values, bind);
            self.get(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }
    }

    #[wasm_bindgen]
    pub fn create_gauge(name: &str, description: &str) -> JsResult<SimpleGauge> {
        ok_or_jserr!(SimpleGauge::new(name, description))
    }

    #[wasm_bindgen]
    pub fn create_multi_gauge(name: &str, description: &str, labels: Vec<JsString>) -> JsResult<MultiGauge> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(MultiGauge::new(name, description, bind.as_slice()))
    }

    #[wasm_bindgen]
    impl MultiGauge {
        #[wasm_bindgen(js_name = "increment_by")]
        pub fn _increment_by(&self, label_values: Vec<JsString>, by: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.increment(bind.as_slice(), by);
        }

        #[wasm_bindgen(js_name = "increment")]
        pub fn _increment(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.increment(bind.as_slice(), 1.0)
        }

        #[wasm_bindgen(js_name = "decrement_by")]
        pub fn _decrement_by(&self, label_values: Vec<JsString>, by: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.decrement(bind.as_slice(), by);
        }

        #[wasm_bindgen(js_name = "decrement")]
        pub fn _decrement(&self, label_values: Vec<JsString>) {
            convert_from_jstrvec!(label_values, bind);
            self.decrement(bind.as_slice(), 1.0)
        }

        #[wasm_bindgen(js_name = "set")]
        pub fn _set(&self, label_values: Vec<JsString>, value: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.set(bind.as_slice(), value);
        }

        #[wasm_bindgen(js_name = "get")]
        pub fn _get(&self, label_values: Vec<JsString>) -> Result<f64, JsValue> {
            convert_from_jstrvec!(label_values, bind);
            self.get(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }
    }

    #[wasm_bindgen]
    pub fn create_histogram(name: &str, description: &str) -> Result<SimpleHistogram, JsValue> {
        create_histogram_with_buckets(name, description, &[] as &[f64; 0])
    }

    #[wasm_bindgen]
    pub fn create_histogram_with_buckets(name: &str, description: &str, buckets: &[f64]) -> JsResult<SimpleHistogram> {
        ok_or_jserr!(SimpleHistogram::new(name, description, buckets.into()))
    }

    #[wasm_bindgen]
    impl SimpleHistogram {
        #[wasm_bindgen(js_name = "start_measure")]
        pub fn wasm_start_measure(&self) -> SimpleTimer {
            SimpleTimer {
                inner: Wasm {
                    start_ts: js_sys::Date::now() / 1000.0,
                    new_ts: || js_sys::Date::now() / 1000.0,
                    labels: vec![],
                },
            }
        }
    }

    #[wasm_bindgen]
    pub fn create_multi_histogram(name: &str, description: &str, labels: Vec<JsString>) -> JsResult<MultiHistogram> {
        create_multi_histogram_with_buckets(name, description, &[] as &[f64; 0], labels)
    }

    #[wasm_bindgen]
    pub fn create_multi_histogram_with_buckets(
        name: &str,
        description: &str,
        buckets: &[f64],
        labels: Vec<JsString>,
    ) -> JsResult<MultiHistogram> {
        convert_from_jstrvec!(labels, bind);
        ok_or_jserr!(MultiHistogram::new(name, description, buckets.into(), bind.as_slice()))
    }

    #[wasm_bindgen]
    impl MultiHistogram {
        #[wasm_bindgen(js_name = "observe")]
        pub fn _observe(&self, label_values: Vec<JsString>, value: f64) {
            convert_from_jstrvec!(label_values, bind);
            self.observe(bind.as_slice(), value)
        }

        #[wasm_bindgen(js_name = "start_measure")]
        pub fn wasm_start_measure(&self, label_values: Vec<JsString>) -> JsResult<SimpleTimer> {
            convert_from_jstrvec!(label_values, bind);
            match self.hh.get_metric_with_label_values(bind.as_slice()) {
                Ok(_) => Ok(SimpleTimer {
                    inner: Wasm {
                        start_ts: js_sys::Date::now() / 1000.0,
                        new_ts: || js_sys::Date::now() / 1000.0,
                        labels: label_values.into_iter().map(String::from).collect(),
                    },
                }),
                Err(x) => Err(JsValue::from(x.to_string())),
            }
        }

        #[wasm_bindgen(js_name = "get_sample_count")]
        pub fn _get_sample_count(&self, label_values: Vec<JsString>) -> JsResult<u64> {
            convert_from_jstrvec!(label_values, bind);
            self.get_sample_count(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }

        #[wasm_bindgen(js_name = "get_sample_sum")]
        pub fn _get_sample_sum(&self, label_values: Vec<JsString>) -> JsResult<f64> {
            convert_from_jstrvec!(label_values, bind);
            self.get_sample_sum(bind.as_slice())
                .ok_or(JsValue::from_str("label value does not exist"))
        }
    }
}
