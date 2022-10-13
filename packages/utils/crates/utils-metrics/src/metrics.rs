use std::fmt::Display;
use prometheus::{Gauge, Histogram, HistogramOpts, HistogramTimer, IntCounter, Opts, TextEncoder};
use prometheus::core::{Collector, Metric};
use wasm_bindgen::JsValue;

pub fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
    JsValue::from(v.to_string())
}

fn register_metric<M, C>(name: &str, desc: &str, creator: C) -> Result<M, String>
where
    M: Metric + Collector + 'static,
    C: Fn(Opts) -> prometheus::Result<M>
{
    let metric = creator(Opts::new(name, desc))
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

struct SimpleHistogram {
    name: String,
    hh: Histogram
}

struct SimpleTimer {
    histogram_timer: HistogramTimer
}

impl SimpleHistogram {
    pub fn new(name: &str, description: &str) -> Result<Self, String> {
        let metric = Histogram::with_opts(HistogramOpts::new(name, description))
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
    use crate::metrics::as_jsvalue;

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
    pub struct SimpleHistogram {
        w: super::SimpleHistogram
    }

    #[wasm_bindgen]
    pub fn create_histogram(name: &str, description: &str) -> Result<SimpleHistogram, JsValue> {
        super::SimpleHistogram::new(name, description)
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



