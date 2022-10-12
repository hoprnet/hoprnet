use std::fmt::Display;
use prometheus::{Encoder, Gauge, Histogram, HistogramOpts, HistogramTimer, IntCounter, Opts, TextEncoder};
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
        register_metric(name, description, |opts| IntCounter::with_opts(opts))
            .map(|m| Self {
                name: name.to_string(),
                ctr: m
            })
    }

    pub fn increment(&self) {
        self.ctr.inc()
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
        register_metric(name, description, |opts| Gauge::with_opts(opts))
            .map(|m| Self {
                name: name.to_string(),
                gg: m
            })
    }

    pub fn increment(&self) {
        self.gg.inc()
    }

    pub fn decrement(&self) {
        self.gg.dec()
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
fn gather_all_metrics() -> Result<Box<[u8]>, String> {

    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    // Simply gather all global metric families and encode them
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|e| e.to_string())?;

    Ok(buffer.into_boxed_slice())
}

/// Bindings for JS/TS
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;
    use crate::metrics::as_jsvalue;

    #[wasm_bindgen]
    pub struct Counter {
        w: super::SimpleCounter
    }

    #[wasm_bindgen]
    pub fn create_counter(name: &str, description: &str) -> Result<Counter, JsValue> {
        super::SimpleCounter::new(name, description)
            .map(|c| Counter { w: c })
            .map_err(as_jsvalue)
    }

    #[wasm_bindgen]
    impl Counter {
        pub fn increment(&self) {
            self.w.increment();
        }

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
        pub fn increment(&self) {
            self.w.increment();
        }

        pub fn decrement(&self) {
            self.w.decrement();
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
    pub fn gather_all_metrics() -> Result<Box<[u8]>, JsValue> {
        super::gather_all_metrics()
            .map_err(as_jsvalue)
    }
}



