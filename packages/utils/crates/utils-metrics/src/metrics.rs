use std::fmt::Display;
use prometheus::{Gauge, GaugeVec, Histogram, HistogramOpts, HistogramTimer, IntCounter, IntCounterVec, Opts, TextEncoder};
use prometheus::core::Collector;
use wasm_bindgen::JsValue;

pub fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
    JsValue::from(v.to_string())
}

fn register_metric<M, C>(name: &str, desc: &str, creator: C) -> Result<M, String>
where
    M: Clone + Collector + 'static,
    C: Fn(Opts) -> prometheus::Result<M>
{
    let metric = creator(Opts::new(name, desc))
        .map_err(|e| e.to_string())?;

    prometheus::register(Box::new(metric.clone()))
        .map_err(|e| e.to_string())?;

    Ok(metric)
}

fn register_metric_vec<M, C>(name: &str, desc: &str, labels: &[&str], creator: C) -> Result<M, String>
    where
        M: Clone + Collector + 'static,
        C: Fn(Opts,&[&str]) -> prometheus::Result<M>
{
    if labels.len() == 0 {
        return Err("at least a single label must be specified".into());
    }

    let metric = creator(Opts::new(name, desc), labels)
        .map_err(|e| e.to_string())?;

    prometheus::register(Box::new(metric.clone()))
        .map_err(|e| e.to_string())?;

    Ok(metric)
}

/// Wrapper for IntCounter metrics type
struct SimpleCounter {
    name: String,
    ctr: IntCounter
}

impl SimpleCounter {
    pub fn new(name: &str, description: &str) -> Result<Self, String> {
        register_metric(name, description, IntCounter::with_opts)
            .map(|m| Self {
                name: name.to_string(),
                ctr: m
            })
    }

    pub fn increment(&self, by: u64) {
        self.ctr.inc_by(by)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

struct MultiCounter {
    name: String,
    ctr: IntCounterVec
}

impl MultiCounter {
    pub fn new(name: &str, description: &str, labels: &[&str]) -> Result<Self, String> {
        register_metric_vec(name, description, labels, IntCounterVec::new)
            .map(|m| Self {
                name: name.to_string(),
                ctr: m
            })
    }

    pub fn increment(&self, label_values: &[&str], by: u64) {
        self.ctr.with_label_values(label_values).inc_by(by)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Wrapper for Gauge metrics type
struct SimpleGauge {
    name: String,
    gg: Gauge
}

impl SimpleGauge {
    pub fn new(name: &str, description: &str) -> Result<Self, String> {
        register_metric(name, description, Gauge::with_opts)
            .map(|m| Self {
                name: name.to_string(),
                gg: m
            })
    }

    pub fn increment(&self, by: f64) {
        self.gg.add(by)
    }

    pub fn decrement(&self, by: f64) {
        self.gg.sub(by)
    }

    pub fn set(&self, value: f64) {
        self.gg.set(value)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

struct MultiGauge {
    name: String,
    ctr: GaugeVec
}

impl MultiGauge {
    pub fn new(name: &str, description: &str, labels: &[&str]) -> Result<Self, String> {
        register_metric_vec(name, description, labels, GaugeVec::new)
            .map(|m| Self {
                name: name.to_string(),
                ctr: m
            })
    }

    pub fn increment(&self, label_values: &[&str], by: f64) {
        self.ctr.with_label_values(label_values).add(by)
    }

    pub fn decrement(&self, label_values: &[&str], by: f64) {
        self.ctr.with_label_values(label_values).sub(by)
    }

    pub fn set(&self, label_values: &[&str], value: f64) {
        self.ctr.with_label_values(label_values).set(value)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

struct SimpleHistogram {
    name: String,
    hh: Histogram
}

struct SimpleTimer {
    histogram_timer: HistogramTimer
}

impl SimpleHistogram {
    pub fn new(name: &str, description: &str, buckets: Vec<f64>) -> Result<Self, String> {
        let mut opts = HistogramOpts::new(name, description);
        if !buckets.is_empty() {
            opts = opts.buckets(buckets);
        }

        let metric = Histogram::with_opts(opts)
            .map_err(|e| e.to_string())?;

        prometheus::register(Box::new(metric.clone()))
            .map_err(|e| e.to_string())?;

        Ok(Self {
            name: name.to_string(),
            hh: metric
        })
    }

    pub fn observe(&self, value: f64) {
        self.hh.observe(value)
    }

    pub fn start_measure(&self) -> SimpleTimer {
        SimpleTimer { histogram_timer: self.hh.start_timer() }
    }

    pub fn record_measure(&self, timer: SimpleTimer) {
        timer.histogram_timer.observe_duration()
    }

    pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
        timer.histogram_timer.stop_and_discard()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Gathers all the global Prometheus metrics.
fn gather_all_metrics() -> Result<String, String> {
    // Simply gather all global metric families
    let metric_families = prometheus::gather();

    // ... and encode them
    let encoder = TextEncoder::new();
    encoder.encode_to_string(&metric_families)
        .map_err(|e| e.to_string())
}

/// Bindings for JS/TS
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;
    use js_sys::JsString;
    use crate::metrics::as_jsvalue;

    #[wasm_bindgen]
    pub struct MultiCounter {
        w: super::MultiCounter
    }

    #[wasm_bindgen]
    pub fn create_multi_counter(name: &str, description: &str, labels: Vec<JsString>) -> Result<MultiCounter, JsValue> {
        let aux: Vec<String> = labels.iter().map(String::from).collect();
        let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        super::MultiCounter::new(name, description, bind.as_slice())
            .map(|c| MultiCounter { w: c })
            .map_err(as_jsvalue)
    }

    #[wasm_bindgen]
    impl MultiCounter {
        pub fn increment_by(&self, label_values: Vec<JsString>, by: u64) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.increment(bind.as_slice(), by);
        }

        pub fn increment(&self, label_values: Vec<JsString>) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.increment(bind.as_slice(), 1)
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    #[wasm_bindgen]
    pub struct SimpleCounter {
        w: super::SimpleCounter
    }

    #[wasm_bindgen]
    pub fn create_counter(name: &str, description: &str) -> Result<SimpleCounter, JsValue> {
        super::SimpleCounter::new(name, description)
            .map(|c| SimpleCounter { w: c })
            .map_err(as_jsvalue)
    }

    #[wasm_bindgen]
    impl SimpleCounter {
        pub fn increment_by(&self, by: u64) {
            self.w.increment(by);
        }

        pub fn increment(&self) { self.w.increment(1) }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    #[wasm_bindgen]
    pub struct SimpleGauge {
        w: super::SimpleGauge
    }

    #[wasm_bindgen]
    pub fn create_gauge(name: &str, description: &str) -> Result<SimpleGauge, JsValue> {
        super::SimpleGauge::new(name, description)
            .map(|c| SimpleGauge { w: c })
            .map_err(as_jsvalue)
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

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    #[wasm_bindgen]
    pub struct MultiGauge {
        w: super::MultiGauge
    }

    #[wasm_bindgen]
    pub fn create_multi_gauge(name: &str, description: &str, labels: Vec<JsString>) -> Result<MultiGauge, JsValue> {
        let aux: Vec<String> = labels.iter().map(String::from).collect();
        let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        super::MultiGauge::new(name, description, bind.as_slice())
            .map(|c| MultiGauge { w: c })
            .map_err(as_jsvalue)
    }

    #[wasm_bindgen]
    impl MultiGauge {
        pub fn increment_by(&self, label_values: Vec<JsString>, by: f64) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.increment(bind.as_slice(), by);
        }

        pub fn increment(&self, label_values: Vec<JsString>) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.increment(bind.as_slice(), 1.0)
        }

        pub fn decrement_by(&self, label_values: Vec<JsString>, by: f64) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.decrement(bind.as_slice(), by);
        }

        pub fn decrement(&self, label_values: Vec<JsString>) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.decrement(bind.as_slice(), 1.0)
        }

        pub fn set(&self, label_values: Vec<JsString>, value: f64) {
            let aux: Vec<String> = label_values.iter().map(String::from).collect();
            let bind: Vec<&str> = aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
            self.w.set(bind.as_slice(), value);
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    #[wasm_bindgen]
    pub struct SimpleHistogram {
        w: super::SimpleHistogram
    }

    #[wasm_bindgen]
    pub fn create_histogram(name: &str, description: &str) -> Result<SimpleHistogram, JsValue> {
        create_histogram_with_buckets(name, description, &[] as &[f64; 0])
    }

    #[wasm_bindgen]
    pub fn create_histogram_with_buckets(name: &str, description: &str, buckets: &[f64]) -> Result<SimpleHistogram, JsValue> {
        super::SimpleHistogram::new(name, description, buckets.into())
            .map(|c| SimpleHistogram { w: c })
            .map_err(as_jsvalue)
    }

    #[wasm_bindgen]
    pub struct SimpleTimer {
        w: super::SimpleTimer
    }

    #[wasm_bindgen]
    impl SimpleHistogram {
        pub fn observe(&self, value: f64) {
            self.w.observe(value)
        }

        pub fn start_measure(&self) -> SimpleTimer {
          SimpleTimer { w: self.w.start_measure() }
        }

        pub fn record_measure(&self, timer: SimpleTimer) {
          self.w.record_measure(timer.w)
        }

        pub fn cancel_measure(&self, timer: SimpleTimer) -> f64 {
            self.w.cancel_measure(timer.w)
        }

        pub fn name(&self) -> String {
            self.w.name().into()
        }
    }

    #[wasm_bindgen]
    pub fn gather_all_metrics() -> Result<String, JsValue> {
        super::gather_all_metrics()
            .map_err(as_jsvalue)
    }
}



