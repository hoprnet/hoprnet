use std::error::Error;
use prometheus::{Encoder, Gauge, IntCounter, Opts, TextEncoder};
use prometheus::core::{Collector, Metric};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("General metrics error: {0}")]
    GeneralError(String)
}

fn register_metric<M, C>(name: &str, desc: &str, creator: C) -> Result<M, MetricsError>
where
    M: Metric + Collector + 'static,
    C: Fn(Opts) -> prometheus::Result<M>
{
    let metric = creator(Opts::new(name, desc))
        .map_err(|e| MetricsError::GeneralError(e.to_string()))?;

    prometheus::register(Box::new(metric.clone()))
        .map_err(|e| MetricsError::GeneralError(e.to_string()))?;

    Ok(metric)
}

struct Counter {
    name: String,
    ctr: IntCounter
}

impl Counter {
    pub fn new(name: &str, description: &str) -> Result<Self, MetricsError> {
        register_metric(name, description, |opts| IntCounter::with_opts(opts))
            .map(|m| Self {
                name: name.to_string(),
                ctr: m
            })
    }

    pub fn increment(&self) {
        self.ctr.inc()
    }
}

struct SimpleGauge {
    name: String,
    gg: Gauge
}

impl SimpleGauge {
    pub fn new(name: &str, description: &str) -> Result<Self, MetricsError> {
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

pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;
    use js_sys::Uint8Array;

    #[wasm_bindgen]
    pub struct Counter {
        w: super::Counter
    }


}



